#!/usr/bin/env bash
# Copyright 2026 SuperFedora Contributors
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_DIR/dist"
mkdir -p "$DIST_DIR"

echo "[+] Generating 4K 16-bit SuperFedora Hero & Ambient wallpapers..."
python3 "$REPO_DIR/scripts/generate_wallpapers.py"

TARBALL="$DIST_DIR/superfedora-wallpapers.tar.gz"
echo "[+] Packaging $TARBALL from wallpapers/..."
tar -czf "$TARBALL" -C "$REPO_DIR/wallpapers" .
sha256sum "$TARBALL" > "$TARBALL.sha256"

echo "[OK] Package complete:"
ls -lh "$TARBALL"*
