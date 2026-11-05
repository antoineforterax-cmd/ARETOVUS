//! Create wallet secrets with an explicit ACL before writing any key material.
use std::{
    ffi::c_void,
    fs::File,
    io,
    iter,
    mem,
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr,
};

use windows_sys::Win32::{
    Foundation::{
        GetLastError, LocalFree, ERROR_NO_TOKEN, GENERIC_WRITE, INVALID_HANDLE_VALUE, READ_CONTROL,
    },
    Security::{
        GetAce, GetSecurityDescriptorControl, GetSecurityDescriptorDacl, GetTokenInformation,
        EqualSid, TokenUser, ACCESS_ALLOWED_ACE, ACCESS_ALLOWED_ACE_TYPE, ACL,
        DACL_SECURITY_INFORMATION, SE_DACL_PROTECTED, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    },
    Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        GetSecurityInfo, SDDL_REVISION_1, SE_FILE_OBJECT,
    },
    Storage::FileSystem::{CreateFileW, CREATE_NEW, FILE_ATTRIBUTE_NORMAL},
    System::Threading::{GetCurrentProcess, GetCurrentThread, OpenProcessToken, OpenThreadToken},
};

struct LocalAllocation(*mut c_void);

impl Drop for LocalAllocation {
    fn drop(&mut self) {
        // Both SDDL conversions and GetSecurityInfo allocate through LocalAlloc.
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn current_user_sid() -> io::Result<String> {
    let mut token = ptr::null_mut();
    // Use the effective thread identity when impersonating. Only a thread without
    // a token may fall back to the process identity.
    unsafe {
        if OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut token) == 0 {
            if GetLastError() != ERROR_NO_TOKEN {
                return Err(io::Error::last_os_error());
            }
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        let _token = OwnedHandle::from_raw_handle(token as _);
        let mut needed = 0u32;
        GetTokenInformation(token, TokenUser, ptr::null_mut(), 0, &mut needed);
        if needed < mem::size_of::<TOKEN_USER>() as u32 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid token user size"));
        }
        // TOKEN_USER contains pointers, so a Vec<u8> would not be aligned.
        let mut words = vec![0usize; (needed as usize).div_ceil(mem::size_of::<usize>())];
        if GetTokenInformation(token, TokenUser, words.as_mut_ptr().cast(), needed, &mut needed)
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        let user = &*words.as_ptr().cast::<TOKEN_USER>();
        if user.User.Sid.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "token has no user SID"));
        }
        let mut sid = ptr::null_mut();
        if ConvertSidToStringSidW(user.User.Sid, &mut sid) == 0 {
            return Err(io::Error::last_os_error());
        }
        let _allocation = LocalAllocation(sid.cast());
        let mut len = 0;
        while *sid.add(len) != 0 {
            len += 1;
        }
        String::from_utf16(std::slice::from_raw_parts(sid, len))
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

fn private_descriptor() -> io::Result<LocalAllocation> {
    // P prevents inherited ACEs from a shared parent. SY permits SYSTEM backup;
    // no Everyone, Users, or Administrators ACE is granted by this descriptor.
    let sddl = format!("D:P(A;;FA;;;{})(A;;FA;;;SY)", current_user_sid()?);
    let wide: Vec<u16> = sddl.encode_utf16().chain(iter::once(0)).collect();
    let mut descriptor = ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide.as_ptr(), SDDL_REVISION_1, &mut descriptor, ptr::null_mut(),
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(LocalAllocation(descriptor))
}

unsafe fn allowed_ace(acl: *mut ACL, index: u32) -> io::Result<*const ACCESS_ALLOWED_ACE> {
    let mut ace = ptr::null_mut();
    if unsafe { GetAce(acl, index, &mut ace) } == 0 || ace.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "cannot inspect wallet ACL",
            ));
    }
    let ace = ace.cast::<ACCESS_ALLOWED_ACE>();
    if unsafe { (*ace).Header.AceType } != ACCESS_ALLOWED_ACE_TYPE {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "unexpected wallet ACL entry",
        ));
    }
    Ok(ace)
}

// Validate the ACL the filesystem actually stored, not only the descriptor
// passed to CreateFileW: FAT-like filesystems may silently ignore it.
fn verify_private_acl(file: &File, expected: *mut c_void) -> io::Result<()> {
    unsafe {
        let mut actual = ptr::null_mut();
        let status = GetSecurityInfo(
            file.as_raw_handle() as _,
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            &mut actual,
        );
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if actual.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "wallet ACL unavailable",
            ));
        }
        let _allocation = LocalAllocation(actual);
        let mut control = 0;
        let mut revision = 0;
        if GetSecurityDescriptorControl(actual, &mut control, &mut revision) == 0
            || control & SE_DACL_PROTECTED == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "wallet ACL is inherited",
            ));
        }
        let (mut actual_present, mut expected_present) = (0, 0);
        let (mut actual_acl, mut expected_acl) = (ptr::null_mut(), ptr::null_mut());
        let (mut actual_defaulted, mut expected_defaulted) = (0, 0);
        if GetSecurityDescriptorDacl(
            actual,
            &mut actual_present,
            &mut actual_acl,
            &mut actual_defaulted,
        ) == 0
            || GetSecurityDescriptorDacl(
                expected,
                &mut expected_present,
                &mut expected_acl,
                &mut expected_defaulted,
            ) == 0
            || actual_present == 0
            || expected_present == 0
            || actual_acl.is_null()
            || expected_acl.is_null()
            || (*actual_acl).AceCount != (*expected_acl).AceCount
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "wallet DACL unavailable",
            ));
        }
        let count = u32::from((*expected_acl).AceCount);
        if count != 2 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "unexpected wallet DACL",
            ));
        }
        let mut matched = [false; 2];
        for i in 0..count {
            let got = allowed_ace(actual_acl, i)?;
            let mut found = false;
            for j in 0..count {
                let want = allowed_ace(expected_acl, j)?;
                if !matched[j as usize]
                    && (*got).Mask == (*want).Mask
                    && (*got).Header.AceFlags == (*want).Header.AceFlags
                    && EqualSid(
                        ptr::addr_of!((*got).SidStart).cast_mut().cast(),
                        ptr::addr_of!((*want).SidStart).cast_mut().cast(),
                    ) != 0
                {
                    matched[j as usize] = true;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "wallet DACL changed",
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn create_private_file(path: &Path) -> io::Result<File> {
    let wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "NUL in wallet path"));
    }
    let wide: Vec<u16> = wide.into_iter().chain(iter::once(0)).collect();
    let descriptor = private_descriptor()?;
    let attrs = SECURITY_ATTRIBUTES {
        nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: descriptor.0,
        bInheritHandle: 0,
    };
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_WRITE | READ_CONTROL,
            0,
            &attrs,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            ptr::null_mut(),
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_handle(handle as _) };
    verify_private_acl(&file, descriptor.0)?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::WalletKey;
    use std::{
        fs,
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn secret_files_have_protected_acl_even_in_shared_directory() {
        let root = std::env::temp_dir().join(format!(
            "aretovus-private-windows-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        // Use the locale-independent Everyone SID. This inspects the file ACL;
        // actual access by a second account needs a separate two-user test.
        let acl = Command::new("icacls")
            .arg(&root)
            .arg("/grant")
            .arg("*S-1-1-0:(OI)(CI)F")
            .output()
            .unwrap();
        assert!(
            acl.status.success(),
            "icacls failed: {}",
            String::from_utf8_lossy(&acl.stderr)
        );
        let key = WalletKey::from_seed_bytes([0x48; 32]);
        let plain = root.join("plain.aretkey");
        let encrypted = root.join("encrypted.aretkey");
        let recovery = root.join("recovery.txt");
        key.save_plain_testnet_file(&plain).unwrap();
        key.save_encrypted_testnet_file(&encrypted, "windows ACL regression password")
            .unwrap();
        key.save_recovery_file(&recovery).unwrap();
        let expected = private_descriptor().unwrap();
        for path in [&plain, &encrypted, &recovery] {
            let file = File::open(path).unwrap();
            verify_private_acl(&file, expected.0).unwrap();
            assert!(!fs::read(path).unwrap().is_empty());
            assert!(create_private_file(path).is_err());
        }
        fs::remove_dir_all(root).unwrap();
    }
}
