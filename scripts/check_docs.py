"""Lightweight docs sanity checks; uses only the Python standard library."""
from __future__ import annotations

import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

ROOT = Path(__file__).resolve().parents[1]
failures: list[str] = []

for md in sorted(ROOT.rglob("*.md")):
    if any(part in {"node_modules", "target", ".git"} for part in md.parts):
        continue
    text = md.read_text(encoding="utf-8")
    fences = [line for line in text.splitlines() if line.lstrip().startswith("```")]
    if len(fences) % 2:
        failures.append(f"{md.relative_to(ROOT)}: nombre impair de clôtures de bloc Markdown ({len(fences)})")
    if re.search(r"(?im)^\s*```markdown\s*$", text):
        failures.append(f"{md.relative_to(ROOT)}: wrapper ```markdown superflu qui peut casser le rendu")
    if "□" in text:
        failures.append(f"{md.relative_to(ROOT)}: case à cocher cassée « □ » (utiliser « - [ ] »)")
    if re.search(r"indy-local", text, flags=re.IGNORECASE):
        failures.append(f"{md.relative_to(ROOT)}: référence obsolète au projet indy-local")
    for match in re.finditer(r"!?\[[^\]]*\]\(([^)]+)\)", text):
        target = match.group(1).strip().split()[0].strip("<>")
        parsed = urlsplit(target)
        if parsed.scheme or target.startswith("#") or target.startswith("//"):
            continue
        path = unquote(parsed.path)
        if not path:
            continue
        candidate = (md.parent / path).resolve()
        if not candidate.exists():
            failures.append(f"{md.relative_to(ROOT)}: lien local cassé vers {target}")

for config in (ROOT / "package.json", ROOT / "tsconfig.json", ROOT / "frontend/tsconfig.json", ROOT / "src-tauri/tauri.conf.json", ROOT / "src-tauri/capabilities/default.json"):
    try:
        import json
        json.loads(config.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover - diagnostic path
        failures.append(f"{config.relative_to(ROOT)}: JSON invalide ({exc})")

if failures:
    print("Vérification documentaire : ÉCHEC")
    print("\n".join(f"- {failure}" for failure in failures))
    sys.exit(1)

print(f"Vérification documentaire : OK ({len(list(ROOT.rglob('*.md')))} fichiers Markdown parcourus, liens locaux et blocs contrôlés)")
