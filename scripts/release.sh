#!/usr/bin/env bash
#
# Release helper — validates preconditions, creates a version tag, and pushes it.
#
# Usage:
#   ./scripts/release.sh <version>
#
# Example:
#   ./scripts/release.sh 0.1.0    # creates and pushes tag v0.1.0
#
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <version>  (e.g. 0.1.0)" >&2
  exit 1
fi

VERSION="$1"
TAG="v${VERSION}"

# Must run from repo root
REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

# --- Precondition checks ---

# 1. Clean working tree
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Error: working tree is dirty. Commit or stash changes first." >&2
  exit 1
fi

# 2. Tag must not already exist
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Error: tag $TAG already exists." >&2
  exit 1
fi

# 3. Cargo.toml version must match
CARGO_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ "$CARGO_VERSION" != "$VERSION" ]]; then
  echo "Error: Cargo.toml version is $CARGO_VERSION but releasing $VERSION." >&2
  echo "Update Cargo.toml first." >&2
  exit 1
fi

# 4. CHANGELOG.md must have an entry for this version
if ! grep -q "^## \[v${VERSION}\]" CHANGELOG.md; then
  echo "Error: CHANGELOG.md has no entry for [v${VERSION}]." >&2
  echo "Add a changelog section before releasing." >&2
  exit 1
fi

# 5. Must be on master/main
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" != "master" && "$BRANCH" != "main" ]]; then
  echo "Warning: you are on branch '$BRANCH', not master/main."
  read -rp "Continue anyway? [y/N] " confirm
  if [[ "$confirm" != [yY] ]]; then
    exit 1
  fi
fi

# --- Create and push tag ---

echo "Creating tag $TAG..."
git tag "$TAG"

echo "Pushing tag $TAG to origin..."
git push origin "$TAG"

echo ""
echo "Done! Tag $TAG pushed."
echo "GitHub Actions will now create a release and publish to crates.io."
echo "Track progress: https://github.com/scottroemeschke/cue-sdk-rust/actions"
