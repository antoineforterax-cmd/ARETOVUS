use std::{
    io::{self, Read, Write},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const POLL_INTERVAL: Duration = Duration::from_millis(10);
// The bridge sends one short request into a fresh, empty stdin pipe.
const MAX_REQUEST: usize = 256;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CaptureError {
    Unavailable,
    Timeout,
    InvalidOutput,
    TooLarge,
    Unsuccessful,
}

pub(crate) fn capture(
    mut command: Command,
    request: Option<&[u8]>,
    max_output: usize,
    timeout: Duration,
) -> Result<Vec<u8>, CaptureError> {
    if request.is_some_and(|bytes| bytes.len() > MAX_REQUEST) {
        return Err(CaptureError::Unavailable);
    }
    command
        .stdin(if request.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stderr(Stdio::null());

    let started = Instant::now();
    let (mut child, mut stdout) = spawn(command)?;
    let result = (|| {
        if started.elapsed() >= timeout {
            return Err(CaptureError::Timeout);
        }
        if let Some(bytes) = request {
            child
                .stdin
                .take()
                .ok_or(CaptureError::Unavailable)?
                .write_all(bytes)
                .map_err(|_| CaptureError::Unavailable)?;
        }
        collect(&mut child, &mut stdout, max_output, started, timeout)
    })();
    if result.is_err() {
        drop(stdout);
        stop(&mut child);
    }
    result
}

fn collect(
    child: &mut Child,
    stdout: &mut impl Read,
    max_output: usize,
    started: Instant,
    timeout: Duration,
) -> Result<Vec<u8>, CaptureError> {
    let mut output = Vec::new();
    let mut buffer = [0; 4096];
    let mut eof = false;
    let mut exit = None;
    loop {
        if started.elapsed() >= timeout {
            return Err(CaptureError::Timeout);
        }
        let mut made_progress = false;
        if !eof {
            // Read at most one extra byte to distinguish the exact limit from overflow.
            let room = max_output - output.len();
            let read_limit = buffer.len().min(room.saturating_add(1));
            match stdout.read(&mut buffer[..read_limit]) {
                Ok(0) => eof = true,
                Ok(count) if count > room => return Err(CaptureError::TooLarge),
                Ok(count) => {
                    output.extend_from_slice(&buffer[..count]);
                    made_progress = true;
                }
                Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(CaptureError::InvalidOutput),
            }
        }
        if exit.is_none() {
            exit = child.try_wait().map_err(|_| CaptureError::Unavailable)?;
        }
        if let (true, Some(status)) = (eof, exit) {
            return if status.success() {
                Ok(output)
            } else {
                Err(CaptureError::Unsuccessful)
            };
        }
        if !made_progress {
            thread::sleep(POLL_INTERVAL.min(timeout.saturating_sub(started.elapsed())));
        }
    }
}

fn stop(child: &mut Child) {
    if child.try_wait().is_ok_and(|status| status.is_some()) {
        return;
    }
    if child.kill().is_ok() {
        let _ = child.wait();
    } else {
        let _ = child.try_wait();
    }
}

#[cfg(unix)]
fn spawn(mut command: Command) -> Result<(Child, std::os::unix::net::UnixStream), CaptureError> {
    use std::os::{fd::OwnedFd, unix::net::UnixStream};

    let (reader, writer) = UnixStream::pair().map_err(|_| CaptureError::Unavailable)?;
    reader
        .set_nonblocking(true)
        .map_err(|_| CaptureError::Unavailable)?;
    let child = command
        .stdout(Stdio::from(OwnedFd::from(writer)))
        .spawn()
        .map_err(|_| CaptureError::Unavailable)?;
    // Command retains its own writer until dropped. Only the child should own one.
    drop(command);
    Ok((child, reader))
}

#[cfg(windows)]
fn spawn(mut command: Command) -> Result<(Child, WindowsStdout), CaptureError> {
    let mut child = command
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| CaptureError::Unavailable)?;
    match child.stdout.take() {
        Some(stdout) => Ok((child, WindowsStdout(stdout))),
        None => {
            stop(&mut child);
            Err(CaptureError::InvalidOutput)
        }
    }
}

#[cfg(windows)]
struct WindowsStdout(std::process::ChildStdout);

#[cfg(windows)]
impl Read for WindowsStdout {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        use std::{os::windows::io::AsRawHandle, ptr};
        use windows_sys::Win32::{Foundation::ERROR_BROKEN_PIPE, System::Pipes::PeekNamedPipe};

        if buffer.is_empty() {
            return Ok(0);
        }
        let mut available = 0;
        // SAFETY: stdout keeps the pipe handle open. We request only the count,
        // passing a valid writable pointer for it and null for unused outputs.
        // PeekNamedPipe requires an asynchronous handle to avoid blocking in a
        // multithreaded process. Rust 1.99.1 creates the parent side that way:
        // https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/process/windows/child_pipe.rs#L31-L40
        let ok = unsafe {
            PeekNamedPipe(
                self.0.as_raw_handle(),
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &mut available,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            let err = io::Error::last_os_error();
            return if err.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) {
                Ok(0)
            } else {
                Err(err)
            };
        }
        if available == 0 {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        // The parent is the only reader, so the peeked bytes are still ready.
        let read_limit = buffer.len().min(available as usize);
        self.0.read(&mut buffer[..read_limit])
    }
}

#[cfg(test)]
#[path = "companion_capture_tests.rs"]
mod tests;
