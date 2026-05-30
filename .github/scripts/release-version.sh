#!/usr/bin/env bash
# Apply package version from the git tag that triggered the release workflow.
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

perl -pi -e "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml

if [[ -n "${GITHUB_ENV:-}" ]]; then
  echo "VERSION=${VERSION}" >> "$GITHUB_ENV"
fi
echo "Set Cargo.toml version to ${VERSION}"
