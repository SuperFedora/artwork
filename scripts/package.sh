#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_DIR/dist"
mkdir -p "$DIST_DIR"

TARBALL="$DIST_DIR/superfedora-wallpapers.tar.gz"
echo "Creating $TARBALL from wallpapers/…"
tar -czf "$TARBALL" -C "$REPO_DIR/wallpapers" .
sha256sum "$TARBALL" > "$TARBALL.sha256"

echo "Package complete:"
ls -lh "$TARBALL"*
