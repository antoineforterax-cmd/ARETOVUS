#!/usr/bin/env python3
"""AURORA experimental LAN assistant for macOS/Linux.

Local-only node controller, no wallet keys, firewall changes, or protocol changes.
Needs Python 3; Tkinter is optional (interactive terminal fallback).
"""
from __future__ import annotations

import argparse
import hashlib
import ipaddress
import os
from pathlib import Path
import re
import signal
import stat
import socket
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
NODE = ROOT / "bin" / "aurora-node"
BEACON_PORT = 19446
NODE_PORT = 19444
MAGIC = "AURORA-LAN-TESTNET-2"
RFC1918 = tuple(ipaddress.ip_network(net) for net in
                ("10.0.0.0/8", "172.16.0.0/12", "192.168.0.0/16"))
REVISION = re.compile(r"(?m)^Source commit: ([0-9a-f]{12})\r?$")


def private_ip(raw: str) -> str:
    try:
        ip = ipaddress.IPv4Address(raw)
    except (ValueError, ipaddress.AddressValueError) as exc:
        raise ValueError("Adresse IPv4 invalide") from exc
    if not any(ip in net for net in RFC1918):
        raise ValueError("Adresse IPv4 LAN privee requise (10/8, 172.16/12, 192.168/16)")
    return str(ip)


def read_package(root: Path = ROOT) -> tuple[str, str]:
    """Check all packaged checksums, not publisher authenticity."""
    stamp = (root / "EXPERIMENTAL_TESTNET.txt").read_text(encoding="utf-8")
    match = REVISION.search(stamp)
    if not match:
        raise ValueError("Source commit absent du manifeste")
    lines = (root / "SHA256SUMS.txt").read_text(encoding="utf-8").splitlines()
    seen: set[str] = set()
    for line in lines:
        if not re.fullmatch(r"[0-9a-f]{64}  [A-Za-z0-9_./-]+", line):
            raise ValueError("Ligne SHA256SUMS invalide")
        digest, path = line.split("  ", 1)
        pure = Path(path)
        if pure.is_absolute() or any(part in ("..", ".") for part in pure.parts):
            raise ValueError("Chemin non sur dans le manifeste")
        if path in seen:
            raise ValueError("Doublon SHA256SUMS")
        seen.add(path)
        file_path = root / path
        if not file_path.is_file():
            raise FileNotFoundError("Fichier absent du paquet : " + path)
        if hashlib.sha256(file_path.read_bytes()).hexdigest() != digest:
            raise ValueError("Somme de controle differente : " + path)
    if "bin/aurora-node" not in seen or "bin/aurora-wallet" not in seen:
        raise ValueError("Binaires AURORA absents du manifeste")
    return match.group(1), hashlib.sha256((root / "bin/aurora-node").read_bytes()).hexdigest()


def restore_local_executable_bits(root: Path = ROOT) -> None:
    """ZIP extractors may strip +x; restore only after checksum verification."""
    for name in ("aurora-node", "aurora-wallet"):
        executable = root / "bin" / name
        mode = executable.stat().st_mode
        if not (mode & stat.S_IXUSR):
            executable.chmod(mode | stat.S_IXUSR)


def discover_interfaces() -> list[str]:
    """Only unambiguous RFC1918 addresses on non-tunnel, active interfaces."""
    if sys.platform.startswith("linux"):
        cmd = ["ip", "-o", "-4", "addr", "show", "up"]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=4, check=True)
        items = []
        for line in result.stdout.splitlines():
            m = re.search(r"^\d+:\s+([^:\s]+)\s+inet\s+(\d+\.\d+\.\d+\.\d+)/", line)
            if m and not re.match(r"^(lo|docker|veth|virbr|br-|tun|tap|wg|tailscale)", m.group(1)):
                items.append(m.group(2))
    elif sys.platform == "darwin":
        result = subprocess.run(["ifconfig"], capture_output=True, text=True, timeout=4, check=True)
        items = []
        interface = ""
        active = False
        for line in result.stdout.splitlines():
            if line and not line[0].isspace():
                interface = line.split(":", 1)[0]
                active = ("UP" in line)
            m = re.search(r"^\s+inet\s+(\d+\.\d+\.\d+\.\d+)\b", line)
            if m and active and re.match(r"^(en|eth)", interface):
                items.append(m.group(1))
    else:
        raise RuntimeError("Ce lanceur vise uniquement macOS et Linux")
    clean = set()
    for item in items:
        try:
            clean.add(private_ip(item))
        except ValueError:
            pass
    return sorted(clean)


def sole_lan_ip() -> str:
    ips = discover_interfaces()
    if len(ips) != 1:
        raise RuntimeError(
            "Une seule interface LAN privee doit etre active : trouvees "
            + (", ".join(ips) if ips else "aucune")
            + ". Deconnectez VPN/autres LAN ou utilisez un seul reseau local."
        )
    return ips[0]


def announce(revision: str) -> bytes:
    if not re.fullmatch(r"[0-9a-f]{12}", revision):
        raise ValueError("Revision non valide")
    return f"{MAGIC}|{revision}|{NODE_PORT}".encode("ascii")


def parse_beacon(data: bytes, sender: str, revision: str) -> str | None:
    if len(data) > 128:
        return None
    try:
        sender = private_ip(sender)
        message = data.decode("ascii")
    except (ValueError, UnicodeDecodeError):
        return None
    if message != announce(revision).decode("ascii"):
        return None
    return sender


def udp_receiver() -> socket.socket:
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("", BEACON_PORT))
    s.setblocking(False)
    return s


def udp_sender(source: str) -> socket.socket:
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_BROADCAST, 1)
    s.bind((private_ip(source), 0))
    return s


def reachable(peer: str) -> bool:
    try:
        with socket.create_connection((private_ip(peer), NODE_PORT), timeout=1.5):
            return True
    except OSError:
        return False


def node_args(role: str, local: str, peer: str = "", root: Path = ROOT) -> list[str]:
    local = private_ip(local)
    if role == "A":
        return [str(root / "bin/aurora-node"), "testnet", "serve", "--data-dir",
                str(root / "two-pc/assistant-a"), "--listen", f"{local}:{NODE_PORT}"]
    if role == "B":
        peer = private_ip(peer)
        if peer == local:
            raise ValueError("Impossible de rejoindre son propre ordinateur comme pair")
        return [str(root / "bin/aurora-node"), "testnet", "serve", "--data-dir",
                str(root / "two-pc/assistant-b"), "--listen", "127.0.0.1:19445",
                "--peer", f"{peer}:{NODE_PORT}"]
    raise ValueError("Role invalide")


class Controller:
    def __init__(self, root: Path, revision: str, binary_sha: str, local: str):
        self.root, self.revision, self.binary_sha, self.local = root, revision, binary_sha, local
        self.proc: subprocess.Popen[bytes] | None = None
        self.role = ""
        self.peer = ""
        self.log_stream = None
        self.last_verified = ""

    def start(self, role: str, peer: str = "") -> None:
        if self.proc is not None and self.proc.poll() is None:
            raise RuntimeError("Un noeud est deja actif")
        if role == "B" and not reachable(peer):
            raise RuntimeError("Noeud A inaccessible en TCP 19444 (pare-feu / LAN)")
        command = node_args(role, self.local, peer, self.root)
        logs = self.root / "two-pc" / "logs"
        logs.mkdir(parents=True, exist_ok=True)
        self.log_stream = (logs / f"node-{role.lower()}.log").open("ab")
        try:
            self.proc = subprocess.Popen(
                command, cwd=self.root, stdin=subprocess.DEVNULL,
                stdout=self.log_stream, stderr=subprocess.STDOUT, start_new_session=True
            )
        except Exception:
            self.log_stream.close()
            self.log_stream = None
            raise
        self.role, self.peer = role, peer

    def stop(self) -> bool:
        if self.proc is None or self.proc.poll() is not None:
            return True
        # Send graceful equivalent of Ctrl+C. Never kill -9 or remove .aurora.lock.
        os.killpg(self.proc.pid, signal.SIGINT)
        try:
            self.proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            return False
        finally:
            if self.proc.poll() is not None and self.log_stream is not None:
                self.log_stream.close()
                self.log_stream = None
        return True

    def verify(self, role: str = "") -> str:
        """Read-only node validation after stopping its writer."""
        if self.proc is not None and self.proc.poll() is None:
            raise RuntimeError("Arretez le noeud avant la verification du stockage")
        role = role or self.role
        if role not in ("A", "B"):
            raise ValueError("Choisir d'abord quel noeud verifier : A ou B")
        source = self.root / "two-pc" / ("assistant-a" if role == "A" else "assistant-b")
        if not source.is_dir():
            raise FileNotFoundError("Aucun dossier de noeud encore cree")
        result = subprocess.run(
            [str(self.root / "bin/aurora-node"), "testnet", "verify", "--data-dir", str(source)],
            cwd=self.root, capture_output=True, text=True, timeout=120, check=False,
        )
        if result.returncode:
            raise RuntimeError("Verification echouee. Inspecter les donnees locales avant de poursuivre")
        match = re.search(
            r"verified testnet data: height=\d+ tip=[a-f0-9]{64} known_blocks=\d+ work=[^\r\n]+",
            result.stdout,
        )
        if not match:
            raise RuntimeError("Sortie de verification inattendue ; ne pas conclure au succes")
        self.last_verified = match.group(0)
        return self.last_verified

    def diagnostic(self) -> Path:
        destination = self.root / "two-pc" / "assistant-report.txt"
        destination.parent.mkdir(parents=True, exist_ok=True)
        # No private IP, username, password, secret, or absolute user paths.
        running = self.proc is not None and self.proc.poll() is None
        lines = [
            "AURORA LAN - diagnostic local (pas une preuve de convergence)",
            f"source_revision={self.revision}",
            f"node_sha256={self.binary_sha}",
            f"role={self.role or 'none'}",
            f"running={str(running).lower()}",
            "private_addresses=redacted",
            "status=experimental; no real funds",
            "final_validation=run testnet verify on each stopped node and compare height/tip/work",
            "verified_snapshot=" + (self.last_verified or "not_run"),
        ]
        destination.write_text("\n".join(lines) + "\n", encoding="utf-8")
        return destination


def wait_discovery(revision: str, own: str, seconds: int = 12) -> str | None:
    receiver = udp_receiver()
    try:
        until = time.monotonic() + seconds
        while time.monotonic() < until:
            try:
                payload, address = receiver.recvfrom(256)
            except BlockingIOError:
                time.sleep(0.15)
                continue
            found = parse_beacon(payload, address[0], revision)
            if found and found != own:
                return found
        return None
    finally:
        receiver.close()


def launch_cli(controller: Controller) -> int:
    print("AURORA - Assistant LAN macOS/Linux, testnet uniquement.")
    print("Paquet verifie localement. Decouverte UDP non authentifiee.")
    print("[1] PC A (serveur)  [2] PC B (rejoindre A)  [3] Rapport  [4] Verifier noeud A/B arrete  [0] Quitter")
    choice = input("Choix : ").strip()
    if choice == "3":
        print("Rapport : ", controller.diagnostic())
        return 0
    if choice == "4":
        role = input("Noeud a verifier (A/B, a l'arret) : ").strip().upper()
        print(controller.verify(role))
        print("Rapport : ", controller.diagnostic())
        return 0
    if choice not in ("1", "2"):
        return 0
    peer = ""
    if choice == "2":
        print("Recherche LAN 12 secondes...")
        peer = wait_discovery(controller.revision, controller.local) or ""
        if not peer:
            peer = input("IPv4 privee de PC A : ").strip()
        peer = private_ip(peer)
        print(f"Pair trouve : {peer}. Les annonces ne prouvent pas son identite.")
        if input("Confirmer connexion vers ce LAN ? (oui/non) : ").strip().lower() != "oui":
            return 1
    controller.start("A" if choice == "1" else "B", peer)
    sender = udp_sender(controller.local) if choice == "1" else None
    print("Noeud lance, Ctrl+C pour arreter proprement. Logs dans two-pc/logs/")
    try:
        while controller.proc is not None and controller.proc.poll() is None:
            if sender:
                sender.sendto(announce(controller.revision), ("255.255.255.255", BEACON_PORT))
            time.sleep(2)
    except KeyboardInterrupt:
        if not controller.stop():
            print("Noeud non arrete sous 10 s ; controle manuel requis.")
            return 1
    finally:
        if sender:
            sender.close()
    print("Rapport : ", controller.diagnostic())
    return 0


def launch_gui(controller: Controller) -> int:
    import tkinter as tk
    from tkinter import messagebox
    app = tk.Tk()
    app.title("AURORA - Centre local de test (experimental)")
    app.geometry("680x485")
    app.resizable(False, False)
    state = tk.StringVar(value=f"Paquet verifie, revision {controller.revision} ; LAN {controller.local}")
    peer = tk.StringVar(value="")
    root_frame = tk.Frame(app, padx=20, pady=14)
    root_frame.pack(fill="both", expand=True)
    tk.Label(root_frame, text="AURORA · connexion LAN", font=("Arial", 17, "bold")).pack(anchor="w")
    tk.Label(root_frame, text="Testnet uniquement. Aucun fond reel. Pare-feu et routeur inchanges.",
             fg="red").pack(anchor="w", pady=(0, 12))
    tk.Label(root_frame, text="PC A heberge un noeud, PC B recherche A sur le meme reseau prive.").pack(anchor="w")
    tk.Label(root_frame, textvariable=state, wraplength=630, justify="left").pack(anchor="w", pady=10)
    row = tk.Frame(root_frame)
    row.pack(anchor="w", pady=4)
    wanted = {"active": False}
    receiver = udp_receiver()
    sender = udp_sender(controller.local)
    last_send = [0.0]

    def warn(err: Exception | str) -> None:
        messagebox.showwarning("AURORA", str(err))

    def start(role: str) -> None:
        try:
            target = peer.get().strip() if role == "B" else ""
            if role == "B":
                target = private_ip(target)
                if not messagebox.askyesno("Connexion LAN",
                       f"Connecter B a {target}:19444 ?\nL'annonce UDP n'authentifie pas le PC distant."):
                    return
            controller.start(role, target)
            wanted["active"] = False
            state.set(f"Noeud {role} lance. Ctrl+C equivalent via bouton Arreter. Voir logs locaux.")
        except Exception as err:
            warn(err)

    def discover() -> None:
        wanted["active"] = True
        state.set("Recherche du PC A sur le LAN... Si UDP bloque, entrer l'IPv4 privee.")

    tk.Button(row, text="Demarrer PC A", width=19, command=lambda: start("A")).pack(side="left", padx=(0, 9))
    tk.Button(row, text="Rechercher PC A", width=19, command=discover).pack(side="left")
    tk.Label(root_frame, text="IPv4 de PC A (decouverte ou saisie manuelle) :").pack(anchor="w", pady=(10, 2))
    tk.Entry(root_frame, textvariable=peer, width=28).pack(anchor="w")
    buttons = tk.Frame(root_frame)
    buttons.pack(anchor="w", pady=8)
    tk.Button(buttons, text="Connecter PC B", width=19, command=lambda: start("B")).pack(side="left", padx=(0, 9))
    def stop():
        try:
            if controller.stop():
                state.set("Noeud arrete. Comparer les hauteurs/tips/work pour verifier les PC.")
            else:
                warn("Le noeud ne s'est pas arrete sous 10 s. Ne pas supprimer .aurora.lock.")
        except Exception as err:
            warn(err)
    tk.Button(buttons, text="Arreter mon noeud", width=19, command=stop).pack(side="left")
    def report():
        try:
            dest = controller.diagnostic()
            state.set("Rapport local cree : two-pc/assistant-report.txt (sans IP ni secrets).")
            messagebox.showinfo("Rapport AURORA", f"Rapport : {dest}")
        except Exception as err:
            warn(err)
    tk.Button(root_frame, text="Exporter un diagnostic local", command=report).pack(anchor="w", pady=5)
    def verify_node():
        try:
            role = controller.role or ("A" if messagebox.askyesno(
                "Verification du noeud", "Verifier le dossier de PC A ? (Non = PC B)"
            ) else "B")
            result = controller.verify(role)
            state.set(result)
            messagebox.showinfo("Verification locale", result)
        except Exception as err:
            warn(err)
    tk.Button(root_frame, text="Verifier le stockage (noeud arrete)", command=verify_node).pack(anchor="w")
    tk.Label(root_frame, text="PREVU (non implemente) : minage continu · sauvegardes guidees · "
             "wallet GUI · performances · rapports inter-PC", wraplength=640,
             fg="#666666", justify="left").pack(anchor="w", pady=9)
    tk.Label(root_frame, text="Ce panneau n'authentifie pas le pair et ne prouve pas la convergence.",
             fg="#666666").pack(anchor="w")

    def tick():
        if controller.role == "A" and controller.proc and controller.proc.poll() is None:
            if time.monotonic() - last_send[0] >= 2:
                try:
                    sender.sendto(announce(controller.revision), ("255.255.255.255", BEACON_PORT))
                except OSError:
                    state.set("Annonces UDP bloquees ; utiliser l'adresse IP manuelle.")
                last_send[0] = time.monotonic()
        for _ in range(12):
            try:
                packet, addr = receiver.recvfrom(256)
            except BlockingIOError:
                break
            candidate = parse_beacon(packet, addr[0], controller.revision)
            if candidate and candidate != controller.local:
                peer.set(candidate)
                if wanted["active"]:
                    wanted["active"] = False
                    state.set(f"PC A detecte : {candidate}. Confirmer via Connecter PC B.")
        if controller.proc and controller.proc.poll() is not None and controller.role:
            state.set(f"Noeud termine (code {controller.proc.returncode}). Verifier les logs.")
        app.after(800, tick)

    def close():
        if controller.proc and controller.proc.poll() is None:
            if not messagebox.askyesno("Noeud actif",
                   "Le noeud tourne encore. Arreter d'abord le noeud ?"):
                return
            if not controller.stop():
                warn("Noeud non arrete. Interface conservee pour eviter un arret brutal.")
                return
        sender.close()
        receiver.close()
        app.destroy()
    app.protocol("WM_DELETE_WINDOW", close)
    app.after(1000, tick)
    app.mainloop()
    return 0


def self_test() -> None:
    assert private_ip("192.168.1.5") == "192.168.1.5"
    for address in ("8.8.8.8", "127.0.0.1", "169.254.10.20", "::1", "192.168.1.1;rm"):
        try:
            private_ip(address)
        except ValueError:
            pass
        else:
            raise AssertionError("Accepted unsafe IP: " + address)
    r = "a" * 12
    assert parse_beacon(announce(r), "10.0.0.1", r) == "10.0.0.1"
    assert parse_beacon(announce(r), "8.8.8.8", r) is None
    assert parse_beacon(announce(r), "10.0.0.1", "b" * 12) is None
    assert node_args("A", "10.0.0.2")[-1] == "10.0.0.2:19444"
    assert node_args("B", "10.0.0.2", "10.0.0.1")[-1] == "10.0.0.1:19444"
    try:
        node_args("B", "10.0.0.2", "10.0.0.2")
    except ValueError:
        pass
    else:
        raise AssertionError("Accepted self-peer")
    print("lan_assistant_selftest=ok")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--verify-package", action="store_true")
    parser.add_argument("--terminal", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        self_test()
        return 0
    try:
        revision, sha = read_package()
        print(f"package_verified=yes source_revision={revision} node_sha256={sha}")
        if args.verify_package:
            return 0
        restore_local_executable_bits()
        controller = Controller(ROOT, revision, sha, sole_lan_ip())
        if not args.terminal:
            try:
                return launch_gui(controller)
            except Exception as exc:  # Tkinter missing, no display, or unavailable display server
                print(f"Interface graphique indisponible ({exc}); mode terminal.", file=sys.stderr)
        return launch_cli(controller)
    except (OSError, ValueError, RuntimeError, subprocess.SubprocessError) as exc:
        print("AURORA assistant: " + str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
