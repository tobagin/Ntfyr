---
name: release
description: Automate semantic releases for Ntfyr by analyzing changes since last tag, bumping version everywhere, and publishing tag to trigger Flathub.
---

# Release

Automate a full Ntfyr release. Use when the user asks to cut a release, bump version, or publish to Flathub. Never run commits, tags, or pushes unless the user explicitly invoked this skill in this session.

## Preconditions

- Working tree clean (`git status` shows no unrelated unstaged changes; if dirty, stop and report).
- On `main` and up to date with `origin/main` (`git fetch` then compare).
- `git describe --tags --abbrev=0` returns last release (e.g. `v0.7.1`); if no tag exists, treat base as `v0.0.0`.

## Step 1 — Analyze Scope Since Last Release

1. Last tag: `LAST=$(git describe --tags --abbrev=0)` and `LAST_COMMIT=$(git rev-list -n 1 $LAST)`; also note `git tag --sort=-v:refname | head`.
2. Collect changes:
   ```bash
   git log $LAST..HEAD --oneline --no-merges
   git log $LAST..HEAD --pretty=format:"%h %s%n%b"   # for BREAKING CHANGE
   git diff $LAST..HEAD --stat
   ```
3. Decide bump per SemVer (document reasoning):
   - **Major** (`x.0.0`): commit body contains `BREAKING CHANGE:` or `!:` trailing `!` (e.g. `feat!:`/`fix!:`), or diff removes/renames GSettings keys, D-Bus APIs, or message-repo migrations that break existing installs.
   - **Minor** (`0.x.0`): at least one `feat:` (new UI, new filter capability, new language, notable UX) and no major signal.
   - **Patch** (`0.0.x`): only `fix:` / `docs:` / `chore:` / `chore(deps):` / `perf:` and dependency bumps. This is the default when uncertain.
4. Compute `NEW_VERSION` (strip leading `v` from `LAST`, bump chosen segment, reset lower segments to 0). Never invent `NEW_VERSION`; derive it. Show `LAST -> NEW_VERSION (Patch|Minor|Major — reason)` and ask for confirmation before writing files if the skill is run interactively; if user already approved the tier, proceed.

## Step 2 — Bump Version Everywhere

Use exact file targets; use `muse.edit_file`/`muse.write_file` with narrow `find` strings. Do not use broad search-replace.

- **Meson** `meson.build:4` — `version: 'NEW_VERSION'`
- **Cargo** `Cargo.toml:3` — `version = "NEW_VERSION"` (workspace root only; `ntfy-daemon/Cargo.toml` stays `0.1.0` — it is an internal crate)
- After `Cargo.toml` edit, run `CARGO_HOME=/tmp/cargo-home cargo update --workspace` if you changed the version to refresh `Cargo.lock` (does not bump unrelated deps unless you explicitly ran `cargo update`), then `git diff Cargo.lock` to verify.
- **Cargo sources** — immediately after any `Cargo.lock` change (including `cargo update` above), regenerate vendored sources **before any commit/tag** so the Flatpak build sees them:
  ```bash
  python3 tools/flatpak-cargo-generator.py Cargo.lock -o packaging/cargo-sources.json
  ```
  Verify with `git diff --stat` and `head -n 5 packaging/cargo-sources.json`.
- **AppData** `data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in:55` — prepend a new `<release version="NEW_VERSION" date="YYYY-MM-DD">` block (use today `date -I` in UTC). Keep history sorted descending; copy description style from previous entry (grouped `<ul><li>`).
- **READMEs** — update all language variants:
  - `README.md:16` `## 🎉 Version NEW_VERSION` and `README.md:31` `### 🆕 What's New in NEW_VERSION` + `### Also in PREV_VERSION` (move previous What's New down). Mirror in `README.de.md`, `README.es.md`, `README.fr.md`, `README.pt.md`, `README.ru.md` with their localized headings (`## Aktuelles Release: NEW_VERSION`, `## Última versión: NEW_VERSION`, etc.).
  - Keep the bullet style consistent; do not hard-code untranslated English into localized files — reuse the `CHANGELOG.md` entry wording translated appropriately or keep concise English fallback and note it.
- **CHANGELOG.md** — insert new section at top after header (after line 6): `## [NEW_VERSION] - YYYY-MM-DD` with Keep-a-Changelog groups (`### ✨ Added`, `### 🐛 Fixed`, `### 🔒 Security`, `### 🔧 Changed`) derived from commits since `LAST`. Each bullet should reference PR/issue numbers when present in commit messages. Do not delete old entries.
- **About dialog** — no direct edit needed: `src/config.rs.in:7-9` uses `meson.build` `version` via `src/meson.build:5-6` (`VERSION = version + version_suffix`, `RELEASE_VERSION = version`). The generated `src/config.rs` is overwritten by `meson setup`/`flatpak-builder`; do not hand-edit generated `src/config.rs`.

Verify each edit:
```bash
grep -n "version.*NEW_VERSION" meson.build Cargo.toml
grep -n "release version" data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in | head
head -n 35 README.md CHANGELOG.md
```

## Step 3 — Commit Release

Git safety: the skill caller has explicitly asked for release commits/tags/pushes, so history writes are authorized for the exact steps below. Name files explicitly; never `git add -A` in a tree with unrelated untracked files.

1. Ensure no lock file blocks (`git status` succeeds; if `.git/index.lock` exists, wait and report, do not delete).
2. Stage versioned files (**include regenerated `packaging/cargo-sources.json` when `Cargo.lock` changed — the tag must contain the new sources or Flatpak will miss them**):
   ```bash
   git add meson.build Cargo.toml Cargo.lock packaging/cargo-sources.json data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in README.md README.de.md README.es.md README.fr.md README.pt.md README.ru.md CHANGELOG.md
   ```
   If `Cargo.lock`/`cargo-sources.json` were not changed, omit them.
3. Commit:
   ```bash
   git commit -m "Release vNEW_VERSION"
   ```
   No `Co-Authored-By`, no tool attribution. Commit as `tobagin`.

## Step 4 — Update Production Flatpak Manifest, Tag, and Push

This is a second, distinct commit so the prod manifest points at the new tag's commit with a verifiable hash (matches existing `d02e3aa` pattern).

1. Get new commit: `NEW_COMMIT=$(git rev-parse HEAD)`
2. Edit `packaging/io.github.tobagin.Ntfyr.yml:52-53`:
   ```yaml
   tag: vNEW_VERSION
   commit: NEW_COMMIT
   ```
3. Verify `packaging/cargo-sources.json` is up to date with `Cargo.lock` **before tagging** (regenerate if stale from Step 3):
   ```bash
   python3 tools/flatpak-cargo-generator.py Cargo.lock -o packaging/cargo-sources.json
   git diff --stat  # should show no diff if already regenerated in Step 2
   ```
   If regenerated here, it must be included in the manifest commit so the tag contains it.
4. Stage and commit (tag must contain the regenerated sources):
   ```bash
   git add packaging/io.github.tobagin.Ntfyr.yml packaging/cargo-sources.json
   git commit -m "packaging: point production manifest at Ntfyr vNEW_VERSION"
   ```
   Omit `cargo-sources.json` from the add only if `git diff` showed no change and it was already committed in Step 3.
5. Tag:
   ```bash
   git tag -a vNEW_VERSION -m "Release vNEW_VERSION"
   ```
6. Push (atomic):
   ```bash
   git push origin main
   git push origin vNEW_VERSION
   ```
   GitHub Actions `Update Flathub on Tag` will open the Flathub PR; do not push to Flathub directly.

## Safety and Recovery

- If `git status` shows unrelated dirty files, stop before Step 2 and list them.
- On lock corruption, do not delete `.git/index.lock`; report and wait.
- If the second manifest commit fails due to stale lock, run `git status`, wait for holder, retry once.
- The CHANGELOG date uses UTC `date -I` (YYYY-MM-DD); do not use local time with offset.

## Completion Checklist

- `meson.build`, `Cargo.toml`, `CHANGELOG.md`, all `README*.md`, AppData, and `packaging/io.github.tobagin.Ntfyr.yml` updated
- `Release vNEW_VERSION` commit + `packaging: point production manifest…` commit present
- Annotated tag `vNEW_VERSION` points at the manifest commit
- `git push origin main && git push origin vNEW_VERSION` succeeded
