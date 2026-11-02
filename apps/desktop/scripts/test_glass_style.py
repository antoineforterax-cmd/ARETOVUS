#!/usr/bin/env python3
"""Static safety/accessibility regression guards for the optional desktop skin.

This is not a visual, WebView or WCAG audit. It catches accidental removal of
fallbacks and the introduction of remote CSS resources.
"""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
css = (ROOT / "src" / "style.css").read_text(encoding="utf-8")
ui = (ROOT / "src" / "App.tsx").read_text(encoding="utf-8")
explorer = (ROOT / "src" / "ExplorerPanel.tsx").read_text(encoding="utf-8")

assert css.count("{") == css.count("}"), "Unbalanced CSS blocks"
assert "@import" not in css and "url(" not in css, "No remote fonts/images"
assert "@supports ((backdrop-filter:" in css, "Blur must be feature-gated"
assert "background: var(--surface-strong)" in css, "Opaque-ish fallback missing"
assert "@media (prefers-reduced-transparency: reduce)" in css
assert "@media (prefers-reduced-motion: reduce)" in css
assert "@media (forced-colors: active)" in css
assert ":focus-visible" in css
assert "@media (max-width: 740px)" in css
assert "pointer-events: none;" in css, "Decorative layers must not block UI controls"

# Guard UI semantics during the purely visual refresh: real data, error states,
# explicit testnet warnings and native IPC must remain intact.
for marker in ('invoke<unknown>("read_node_status"', "ExplorerPanel", "TESTNET EXPÉRIMENTAL",
               'role="status"', 'role="alert"', "Pas de fonds réels"):
    assert marker in ui, f"Missing status/identity guard: {marker}"
for marker in ('invoke<unknown>("read_canonical_page"', "<table", "page.next_start",
               "role=\"alert\""):
    assert marker in explorer, f"Missing explorer/feedback guard: {marker}"

# Ensure long hashes still wrap and navigation buttons have visual focus.
assert re.search(r"\.explorer-table code\s*\{[^}]*overflow-wrap:\s*anywhere", css)
assert re.search(r"button:focus-visible[^\{]*\{[^}]*outline:\s*3px", css)
print("Desktop glass theme static contracts passed (not a visual/accessibility audit)")
