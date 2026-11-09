#!/usr/bin/env python3
"""Exercise the extracted Tauri/WebKit UI on a private Xvfb and D-Bus session."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import tempfile
import time

import gi

gi.require_version("Atspi", "2.0")
from gi.repository import Atspi, Gio, GLib

from test_linux_desktop_window import TITLE, verify_window, visible_windows


DATA_FIELD = "Dossier de données ARETOVUS (testnet uniquement)"
CONNECT = "Connecter le nœud"
MISSING = "Dossier introuvable ou inaccessible"
VALIDATED = "Snapshot local validé"
HASH_FIELD = "Hash du bloc"
SEARCH_HASH = "Chercher par hash"
FOUND = "Bloc 1 trouvé dans le snapshot."
INPUT_ROLES = {Atspi.Role.ENTRY, Atspi.Role.TEXT}


def start_accessibility_bus() -> str:
    if not os.environ.get("DBUS_SESSION_BUS_ADDRESS", "").startswith("unix:"):
        raise RuntimeError("Run the smoke test inside a private local D-Bus session")
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    bus.call_sync(
        "org.a11y.Bus", "/org/a11y/bus", "org.freedesktop.DBus.Properties", "Set",
        GLib.Variant("(ssv)", ("org.a11y.Status", "IsEnabled", GLib.Variant("b", True))),
        None, Gio.DBusCallFlags.NONE, 5_000,
    )
    (address,) = bus.call_sync(
        "org.a11y.Bus", "/org/a11y/bus", "org.a11y.Bus", "GetAddress", None,
        GLib.VariantType.new("(s)"), Gio.DBusCallFlags.NONE, 5_000,
    ).unpack()
    if not address.startswith("unix:"):
        raise RuntimeError("Expected a local accessibility bus")
    return address


def descendants(root, deadline):
    pending = [root]
    visited = 0
    while pending:
        if time.monotonic() >= deadline:
            return
        element = pending.pop()
        visited += 1
        if visited > 512:
            raise RuntimeError("The accessibility tree exceeded the lab node limit")
        yield element
        count = element.get_child_count()
        if count > 512:
            raise RuntimeError("The accessibility tree exceeded the lab child limit")
        for index in reversed(range(count)):
            if time.monotonic() >= deadline:
                return
            child = element.get_child_at_index(index)
            if child is not None:
                pending.append(child)


def web_document(pid, deadline):
    desktop = Atspi.get_desktop(0)
    if desktop is None:
        return None
    for index in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(index)
        if app is None or app.get_process_id() != pid:
            continue
        app.clear_cache()
        for element in descendants(app, deadline):
            if element.get_role() == Atspi.Role.FRAME and element.get_name() == TITLE:
                return next((child for child in descendants(element, deadline)
                             if child.get_role() == Atspi.Role.DOCUMENT_WEB), None)
    return None


def labels(element):
    name = element.get_name()
    if name:
        yield name
    text = element.get_text_iface()
    if text is not None:
        length = text.get_character_count()
        if 0 < length <= 1024:
            value = text.get_text(0, length)
            if value:
                yield value


class Desktop:
    def __init__(self, process, window, environment):
        self.process = process
        self.window = window
        self.environment = environment

    def wait(self, description, query):
        deadline = time.monotonic() + 25
        last_error = None
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise RuntimeError(f"Desktop exited while waiting for {description}")
            try:
                document = web_document(self.process.pid, deadline)
                if document is not None:
                    result = query(document, deadline)
                    if result and self.process.poll() is None:
                        return result
            except GLib.Error as error:
                last_error = error
            time.sleep(0.2)
        raise TimeoutError(f"Timed out waiting for {description}") from last_error

    def control(self, name, roles, required_state):
        return self.wait(name, lambda root, deadline: next((
            element for element in descendants(root, deadline)
            if element.get_role() in roles and element.get_name() == name
            and element.get_state_set().contains(required_state)
        ), None))

    def xdotool(self, *arguments):
        return subprocess.run(
            ["/usr/bin/xdotool", *arguments], env=self.environment, check=True,
            capture_output=True, text=True, timeout=5,
        ).stdout.strip()

    def type_text(self, name, value):
        field = self.control(name, INPUT_ROLES, Atspi.StateType.EDITABLE)
        self.xdotool("windowfocus", "--sync", self.window)
        component = field.get_component_iface()
        if component is None or not component.grab_focus():
            raise RuntimeError(f"Could not focus {name}")
        self.control(name, INPUT_ROLES, Atspi.StateType.FOCUSED)
        if self.xdotool("getwindowfocus") != self.window:
            raise RuntimeError("The verified ARETOVUS window lost keyboard focus")
        self.xdotool("key", "--clearmodifiers", "ctrl+a")
        self.xdotool("type", "--clearmodifiers", "--delay", "1", "--", value)

    def click(self, name):
        button = self.control(name, {Atspi.Role.PUSH_BUTTON}, Atspi.StateType.SENSITIVE)
        actions = button.get_action_iface()
        if actions is not None:
            for index in range(actions.get_n_actions()):
                action = actions.get_action_name(index)
                if action and action.lower() in {"click", "press", "activate"}:
                    if actions.do_action(index):
                        return
        raise RuntimeError(f"Could not activate {name} in the WebKit UI")


def shows(root, deadline, text, *, exact=False):
    for element in descendants(root, deadline):
        if element.get_state_set().contains(Atspi.StateType.EDITABLE):
            continue
        if any(text == label.strip() if exact else text in label for label in labels(element)):
            return True
    return False


def shows_hash_in_table(root, deadline, expected_hash):
    return any(
        element.get_role() == Atspi.Role.TABLE and shows(element, deadline, expected_hash, exact=True)
        for element in descendants(root, deadline)
    )


def mine_snapshot(package, snapshot, environment):
    snapshot.mkdir()
    subprocess.run(
        [str(package / "aretovus-node"), "testnet", "mine", "--data-dir", str(snapshot),
         "--miner", "11" * 32, "--count", "1"],
        cwd=snapshot.parent, env=environment, capture_output=True, check=True, timeout=100,
    )
    status = subprocess.run(
        [str(package / "aretovus-local-bridge"), "--data-dir", str(snapshot)],
        input=b'{"schema_version":1,"method":"status"}', cwd=snapshot.parent,
        env=environment, capture_output=True, check=True, timeout=30,
    )
    result = json.loads(status.stdout)
    expected_hash = result.get("tip")
    if (type(result.get("height")) is not int or result["height"] != 1
            or not isinstance(expected_hash, str) or not re.fullmatch(r"[a-f0-9]{64}", expected_hash)):
        raise RuntimeError("The disposable native snapshot did not have a valid height-one tip")
    return expected_hash


def exercise_snapshot(package):
    with tempfile.TemporaryDirectory(prefix="aretovus-linux-snapshot-") as temporary:
        work = Path(temporary)
        home = work / "home"
        runtime = work / "runtime"
        home.mkdir(mode=0o700)
        runtime.mkdir(mode=0o700)
        base_environment = {"PATH": os.defpath, "HOME": str(home), "LANG": "C.UTF-8"}
        snapshot = work / "disposable snapshot with spaces"
        expected_hash = mine_snapshot(package, snapshot, base_environment)

        accessibility_address = start_accessibility_bus()
        os.environ["AT_SPI_BUS_ADDRESS"] = accessibility_address
        Atspi.set_timeout(1_000, 3_000)
        if Atspi.init() not in (0, 1):
            raise RuntimeError("Could not initialize AT-SPI")
        environment = {
            **base_environment,
            "DISPLAY": os.environ["DISPLAY"], "XAUTHORITY": os.environ["XAUTHORITY"],
            "DBUS_SESSION_BUS_ADDRESS": os.environ["DBUS_SESSION_BUS_ADDRESS"],
            "AT_SPI_BUS_ADDRESS": accessibility_address, "XDG_RUNTIME_DIR": str(runtime),
            "GDK_BACKEND": "x11",
        }
        with tempfile.TemporaryFile() as diagnostic:
            process = subprocess.Popen(
                [str(package / "aretovus-desktop")], cwd=home, env=environment,
                stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=diagnostic,
                start_new_session=True,
            )
            try:
                verify_window(process)
                windows = visible_windows(process.pid)
                if len(windows) != 1:
                    raise RuntimeError("Expected one visible window from the extracted desktop process")
                desktop = Desktop(process, windows.pop(), environment)
                desktop.type_text(DATA_FIELD, str(work / "missing snapshot"))
                desktop.click(CONNECT)
                desktop.wait("native rejection of a missing path", lambda root, deadline: shows(root, deadline, MISSING))

                desktop.type_text(DATA_FIELD, str(snapshot))
                desktop.click(CONNECT)
                desktop.wait("validated snapshot and its native tip", lambda root, deadline: (
                    shows(root, deadline, VALIDATED) and shows(root, deadline, expected_hash, exact=True)
                ))

                desktop.type_text(HASH_FIELD, expected_hash)
                desktop.click(SEARCH_HASH)
                desktop.wait("explorer result from the native companion", lambda root, deadline: (
                    shows(root, deadline, FOUND) and shows_hash_in_table(root, deadline, expected_hash)
                ))
                if not visible_windows(process.pid):
                    raise RuntimeError("The extracted desktop window disappeared during the UI exercise")
                print("Verified WebKit rejected a missing directory, displayed the native tip and found its block by hash")
            except Exception as error:
                diagnostic.seek(max(0, diagnostic.seek(0, os.SEEK_END) - 4096))
                detail = diagnostic.read(4096).decode("utf-8", errors="replace").strip()
                raise RuntimeError(f"{error}; desktop stderr tail: {detail}") from error
            finally:
                try:
                    os.killpg(process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                process.wait(timeout=5)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("package", type=Path, help="directory of the verified extracted Linux lab")
    args = parser.parse_args()
    exercise_snapshot(args.package.resolve(strict=True))
