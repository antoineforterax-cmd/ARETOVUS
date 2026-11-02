#!/usr/bin/env python3
"""ARETOVUS V2: offline HTML snapshot from the verified read-only JSON contract.

No web server, JavaScript, remote fonts, wallet secrets, active mining or public
network connection. The HTML is a local snapshot, not a live synchronized UI.
"""
from __future__ import annotations

import argparse
from html import escape
import json
import os
from pathlib import Path
import re
import sys
import tempfile

MAX_JSON_BYTES = 4096
TIP = re.compile(r"[0-9a-f]{64}\Z")
WORK = re.compile(r"ChainWork\(\[[0-9, ]{1,240}\]\)\Z")

DOCUMENT = """<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'; object-src 'none'">
<title>ARETOVUS · État du nœud local</title>
<style>
:root {color-scheme:dark; font-family:system-ui,Segoe UI,Arial,sans-serif;background:#090d16;color:#e7edf9}
* {box-sizing:border-box}
body {margin:0;min-height:100vh;background:radial-gradient(ellipse at 70% 0%,#182943 0%,#090d16 56%);line-height:1.5}
main {max-width:1080px;margin:0 auto;padding:52px 24px 70px}
header {display:flex;align-items:center;gap:14px;margin-bottom:58px;flex-wrap:wrap}
.mark {display:grid;place-items:center;width:44px;height:44px;background:#d3b77b;color:#161d27;border-radius:12px;font-size:28px;font-weight:800}
.brand {font-weight:800;letter-spacing:.25em}
.tag {margin-left:auto;color:#e2c488;font-size:12px;border:1px solid #6b572f;padding:7px 11px;border-radius:100px}
.eyebrow {color:#e2c488;font-size:12px;font-weight:700;letter-spacing:.2em;text-transform:uppercase}
h1 {font-size:clamp(34px,6vw,60px);line-height:1.12;letter-spacing:-.045em;margin:16px 0}
.lead {color:#9eabc1;max-width:650px;font-size:17px;margin-bottom:38px}
.grid {display:grid;grid-template-columns:repeat(3,minmax(0,1fr));gap:16px}
article {border:1px solid #2b3447;background:#111a2a;border-radius:18px;padding:24px;min-width:0}
.label {color:#a8b6cf;font-size:12px;letter-spacing:.12em;text-transform:uppercase}
.value {font-size:clamp(28px,4vw,40px);letter-spacing:-.04em;font-weight:750;margin:10px 0 2px}
.muted {color:#8898af;font-size:13px}
.full {grid-column:1/-1}
.mono {font-family:ui-monospace,SFMono-Regular,Consolas,monospace;font-size:14px;word-break:break-all;white-space:normal}
.pill {display:inline-block;color:#b9edce;border:1px solid #2d7853;background:#153a31;border-radius:99px;padding:4px 11px;font-size:13px;margin-top:12px}
section {margin-top:24px}
section h2 {font-size:21px;letter-spacing:-.02em}
.panel {border:1px solid #344055;border-radius:16px;background:#101729;padding:20px 24px}
.panel p {color:#aab7cd}
footer {margin-top:45px;color:#7b8aa4;font-size:13px;border-top:1px solid #263246;padding-top:22px}
@media(max-width:700px){.grid{grid-template-columns:1fr}.tag{margin-left:0}.full{grid-column:auto}main{padding:24px 16px}}
</style>
</head>
<body><main>
<header><div class="mark" aria-hidden="true">A</div><div class="brand">ARETOVUS</div><span class="tag">TESTNET EXPÉRIMENTAL</span></header>
<div class="eyebrow">Tableau de bord · Instantané local</div>
<h1>État vérifié<br>du nœud.</h1>
<p class="lead">Ce rapport a été généré à partir du nœud Rust local. Il ne se met pas à jour en direct et ne garantit pas une synchronisation avec l'ensemble du réseau.</p>
<div class="grid" aria-label="État de la chaîne">
<article><div class="label">Hauteur locale</div><div class="value">{height}</div><div class="muted">Blocs après la genèse</div></article>
<article><div class="label">Blocs connus</div><div class="value">{known_blocks}</div><div class="muted">Incluant les branches secondaires</div></article>
<article><div class="label">Réseau</div><div class="value">Testnet</div><span class="pill">Validation locale réussie</span></article>
<article class="full"><div class="label">Empreinte du sommet (tip)</div><p class="mono">{tip}</p></article>
<article class="full"><div class="label">Travail cumulé · Diagnostic interne</div><p class="mono">{chain_work}</p><p class="muted">Représentation debug non prévue pour des calculs financiers.</p></article>
</div>
<section class="panel" aria-label="Fonctionnalités du produit">
<h2>Les autres modules arrivent par étapes</h2>
<p>Wallet non custodial, contrôle du minage, explorateur des transactions et gestion des pairs P2P nécessitent le pont local sécurisé. Aucun de ces contrôles n'est simulé dans cette première vue.</p>
</section>
<footer>Source : aretovus-node testnet verify · Rapport local en lecture seule · Pas de clés privées · Pas de fonds réels · Aucune connexion réseau initiée par cette page.</footer>
</main></body></html>
"""


def checked_payload(data: object) -> dict[str, object]:
    if not isinstance(data, dict):
        raise ValueError("expected_object")
    if (data.get("schema_version") != 1 or data.get("status") != "verified"
            or data.get("read_only") is not True or data.get("network") != "testnet"):
        raise ValueError("unverified_state")
    height, known, tip, work = (
        data.get("height"), data.get("known_blocks"),
        data.get("tip"), data.get("chain_work_display"),
    )
    if (type(height) is not int or type(known) is not int
            or not 0 <= height < 2**64 or not height + 1 <= known < 2**64):
        raise ValueError("invalid_counts")
    if not isinstance(tip, str) or not TIP.fullmatch(tip):
        raise ValueError("invalid_tip")
    if not isinstance(work, str) or not WORK.fullmatch(work):
        raise ValueError("invalid_work")
    return data


def render(data: object) -> str:
    status = checked_payload(data)
    # Direct replacement keeps CSS braces literal and substitutes only
    # fixed, explicitly validated placeholders. There is no HTML injection.
    result = DOCUMENT
    values = {
        "{height}": f"{status['height']:,}".replace(",", " "),
        "{known_blocks}": f"{status['known_blocks']:,}".replace(",", " "),
        "{tip}": escape(str(status["tip"])),
        "{chain_work}": escape(str(status["chain_work_display"])),
    }
    for marker, value in values.items():
        result = result.replace(marker, value)
    return result


def write_atomic(path: Path, contents: str) -> None:
    target = path.expanduser().absolute()
    target.parent.mkdir(parents=True, exist_ok=True)
    if target.is_symlink():
        raise ValueError("refuse_symlink_output")
    temp_name = None
    try:
        with tempfile.NamedTemporaryFile(
            mode="w", encoding="utf-8", dir=target.parent,
            prefix=".aretovus-view-", suffix=".tmp", delete=False,
        ) as output:
            temp_name = output.name
            output.write(contents)
            output.flush()
            os.fsync(output.fileno())
        os.replace(temp_name, target)
        temp_name = None
    finally:
        if temp_name is not None:
            os.unlink(temp_name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--status-json", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    try:
        path = args.status_json.expanduser()
        if path.stat().st_size > MAX_JSON_BYTES:
            raise ValueError("status_too_large")
        data = json.loads(path.read_text(encoding="utf-8"))
        result = render(data)
        write_atomic(args.output, result)
    except (OSError, ValueError, UnicodeError) as err:
        # Do not include personal paths or JSON payloads in user-facing logs.
        print("ARETOVUS page generation refused: invalid or unsafe status input", file=sys.stderr)
        return 2
    print("ARETOVUS offline status page generated (no live updates).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
