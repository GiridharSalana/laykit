#!/usr/bin/env bash
# Apply package version from the git tag that triggered the release workflow.
# Usage: release-version.sh   (expects GITHUB_REF=refs/tags/vX.Y.Z)
set -euo pipefail

TAG="${GITHUB_REF#refs/tags/}"
VERSION="${TAG#v}"

if [[ -z "$VERSION" || "$TAG" == "$VERSION" ]]; then
  echo "Expected tag ref like refs/tags/v0.0.5, got: ${GITHUB_REF:-<unset>}"
  exit 1
fi

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]]; then
  echo "Invalid semver in tag: $TAG"
  exit 1
fi

sed -i.bak "s/^version = .*/version = \"${VERSION}\"/" Cargo.toml
rm -f Cargo.toml.bak

# Refresh lockfile entry for this crate (best-effort if Rust is not installed yet).
if command -v cargo >/dev/null 2>&1; then
  cargo generate-lockfile
fi

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "VERSION=${VERSION}" >> "$GITHUB_ENV"
fi
echo "Set Cargo.toml version to ${VERSION}"
