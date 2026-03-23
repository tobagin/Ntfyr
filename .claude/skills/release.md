---
description: Create a new version release by analyzing changes, bumping version, updating changelogs, committing, tagging, and pushing
---

# Release Skill

This skill automates the entire release process for Ntfyr.

## Step 1: Analyze Changes Since Last Release

```bash
# Get the last release tag
git describe --tags --abbrev=0

# List all commits since the last tag
git log $(git describe --tags --abbrev=0)..HEAD --oneline --no-merges

# Check for uncommitted changes
git status --short
```

Categorize all changes into:
- **Added**: New features
- **Changed**: Modifications to existing functionality
- **Fixed**: Bug fixes
- **Removed**: Removed features
- **Breaking**: Breaking changes (triggers major version bump)

## Step 2: Determine Version Bump

Follow [Semantic Versioning](https://semver.org/):
- **MAJOR** (X.0.0): Breaking changes or major rewrites
- **MINOR** (x.Y.0): New features, backward compatible
- **PATCH** (x.y.Z): Bug fixes, metadata updates, backward compatible

Current version is in `meson.build` (line ~2): `version: 'X.Y.Z'`

Ask the user to confirm the version before proceeding.

## Step 3: Update Version Number

Update `meson.build`:
```
version: 'X.Y.Z'
```

## Step 4: Update CHANGELOG.md

Add a new version section directly after the `## [Unreleased]` line. Use emoji headings to match the existing style:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### ✨ New Features

- **Feature Name**: Description.

### 🔧 Changed

- **Thing**: Description.

### 🐛 Fixed

- **Bug**: Description.

### 📰 Metadata & Documentation

- **Thing**: Description.
```

Only include sections that have entries. Reset `## [Unreleased]` to empty after.

## Step 5: Update metainfo.xml.in

Add a new `<release>` entry at the TOP of the `<releases>` section in
`data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in`:

```xml
<release version="X.Y.Z" date="YYYY-MM-DD">
  <description>
    <p>Short release title or summary.</p>
    <ul>
      <li>Feature or fix description</li>
      <li>Feature or fix description</li>
    </ul>
  </description>
</release>
```

- The `<p>` may include an emoji for the title (e.g. `✨ New Feature Release`)
- No emojis inside `<li>` items
- Keep entries concise

## Step 6: Commit All Changes

```bash
git add meson.build CHANGELOG.md \
  data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in
git commit -m "Release vX.Y.Z

Changes in this release:
- [List main changes]
- [One per line]

Files updated:
- meson.build (version bump)
- CHANGELOG.md (release notes)
- metainfo.xml.in.in (AppStream release)"
```

**IMPORTANT**: Do NOT add `Co-Authored-By: Claude` to the commit.

## Step 7: Create and Push Tag

```bash
git tag -a vX.Y.Z -m "Release vX.Y.Z"
git push origin HEAD --tags
```

## Step 8: Verify

- Show the tag and commit hash created
- Confirm push was successful
- Remind user to create a GitHub release: `gh release create vX.Y.Z --generate-notes --repo tobagin/Ntfyr`

## Important Notes

- Always use `vX.Y.Z` format for tags (with `v` prefix)
- Dates use `YYYY-MM-DD` format everywhere
- Never force push tags unless explicitly requested

## File Locations Summary

| File | Purpose |
|------|---------|
| `meson.build` line ~2 | Version number |
| `CHANGELOG.md` | Detailed release notes (emoji section headings) |
| `data/resources/io.github.tobagin.Ntfyr.metainfo.xml.in.in` | AppStream release entry |
