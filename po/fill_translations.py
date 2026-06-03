#!/usr/bin/env python3
"""Fill po translation files from embedded dictionaries."""

from __future__ import annotations

import re
from pathlib import Path

from fill_translations_data import DE, ES, FR, RU

LANG_MAP = {"ru": RU, "de": DE, "es": ES, "fr": FR}


def unescape_po(s: str) -> str:
    return s.replace("\\n", "\n").replace('\\"', '"').replace("\\\\", "\\")


def escape_po(s: str) -> str:
    return s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n")


def read_continuation(lines: list[str], start: int) -> tuple[str, int]:
    parts: list[str] = []
    i = start
    while i < len(lines):
        m = re.match(r'^"(.*)"\s*$', lines[i])
        if not m:
            break
        parts.append(m.group(1))
        i += 1
    return unescape_po("".join(parts)), i


def read_msgid(lines: list[str], start: int) -> tuple[str, int]:
    line = lines[start]
    if line.rstrip() == 'msgid ""':
        if start + 1 < len(lines) and lines[start + 1].startswith("msgstr"):
            return "", start + 1
        text, i = read_continuation(lines, start + 1)
        return text, i
    m = re.match(r'^msgid "(.*)"\s*$', line)
    if m:
        return unescape_po(m.group(1)), start + 1
    return "", start + 1


def skip_msgstr(lines: list[str], start: int) -> int:
    i = start + 1
    if start < len(lines) and lines[start].rstrip() == 'msgstr ""':
        _, i = read_continuation(lines, start + 1)
    return i


def write_msgstr(msgstr: str) -> str:
    return f'msgstr "{escape_po(msgstr)}"\n'


def fill_po(path: Path, table: dict[str, str]) -> None:
    lines = path.read_text(encoding="utf-8").splitlines(keepends=True)
    out: list[str] = []
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("msgid "):
            msgid_start = i
            msgid, msgid_end = read_msgid(lines, i)
            if msgid and msgid_end < len(lines) and lines[msgid_end].startswith("msgstr "):
                msgstr = table.get(msgid)
                if msgstr is None:
                    msgstr = table.get(msgid.replace('\\"', '"'), msgid)
                for j in range(msgid_start, msgid_end):
                    out.append(lines[j])
                out.append(write_msgstr(msgstr))
                i = skip_msgstr(lines, msgid_end)
                continue
        out.append(line)
        i += 1
    text = "".join(out)
    text = text.replace("charset=ASCII", "charset=UTF-8")
    text = re.sub(r"\n#, fuzzy\n", "\n", text)
    path.write_text(text, encoding="utf-8")


def main() -> None:
    root = Path(__file__).parent
    for lang, table in LANG_MAP.items():
        fill_po(root / f"{lang}.po", table)
    print("Filled", ", ".join(LANG_MAP.keys()))


if __name__ == "__main__":
    main()
