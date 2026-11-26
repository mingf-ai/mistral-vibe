#!/usr/bin/env bash
# patch-vibe-rs.sh — build this fork's Rust TUI (vibe/cli-rust) and swap it over
# the installed uv-tool vibe's bundled binary, so the fork's client features
# (today: the footer prompt-cache gauge) run in daily sessions.
#
# Usage:
#   scripts/patch-vibe-rs.sh [ref]     build ref (default: the footer-cache branch)
#                                      and swap it in
#   scripts/patch-vibe-rs.sh --check   show what is installed, change nothing
#   scripts/patch-vibe-rs.sh --revert  restore the stock binary backup
#
# The swap affects only NEW vibe processes: exit and `vibe --continue` to pick
# it up. Any uv tool upgrade (or an accepted `vibe update`) restores the stock
# binary — re-run this script afterwards; it is idempotent and no-ops when the
# installed binary already matches the build.
#
# Environment:
#   VIBE_PATCH_SRC   source checkout (default: ~/git/mistral-vibe, cloned if missing)
#   VIBE_PATCH_REF   default ref    (default: vibe/footer-cache-expiry-695da7)

set -euo pipefail

REF_DEFAULT="vibe/footer-cache-expiry-695da7"
SRC="${VIBE_PATCH_SRC:-$HOME/git/mistral-vibe}"

die() { echo "patch-vibe-rs.sh: $*" >&2; exit 1; }

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | cut -d' ' -f1
  else
    sha256sum "$1" | cut -d' ' -f1
  fi
}

resolve_bindir() {
  local root dir entry
  root="$(uv tool dir 2>/dev/null || echo "$HOME/.local/share/uv/tools")"
  # shellcheck disable=SC2012
  # shellcheck disable=SC2012
  dir="$(ls -d "$root"/mistral-vibe/lib/python*/site-packages/vibe/_bin 2>/dev/null | head -1 || true)"
  if [ -z "$dir" ] && entry="$(command -v vibe || true)" && [ -n "$entry" ]; then
  # shellcheck disable=SC2012
    # shellcheck disable=SC2012
    dir="$(ls -d "$(dirname "$entry")"/../lib/python*/site-packages/vibe/_bin 2>/dev/null | head -1 || true)"
  fi
  [ -n "$dir" ] || die "cannot locate the installed vibe's _bin directory"
  echo "$dir"
}

MODE="patch"
REF="${VIBE_PATCH_REF:-$REF_DEFAULT}"
case "${1:-}" in
  "") ;;
  --check) MODE="check" ;;
  --revert) MODE="revert" ;;
  -*) die "unknown flag: $1 (usage: [--check|--revert] [ref])" ;;
  *) REF="$1" ;;
esac

BINDIR="$(resolve_bindir)"
BIN="$BINDIR/vibe-rs"
STOCK="$BINDIR/vibe-rs.stock"
MARKER="$BINDIR/vibe-rs.patched.sha256"

if [ "$MODE" = check ]; then
  echo "installed: $BIN"
  [ -f "$BIN" ] || die "binary missing"
  echo "  current:  $(sha256_file "$BIN")"
  if [ -f "$MARKER" ]; then echo "  marker:   $(cat "$MARKER")"; else echo "  marker:   (none)"; fi
  if [ -f "$STOCK" ]; then echo "  backup:   $STOCK ($(sha256_file "$STOCK"))"; else echo "  backup:   (none)"; fi
  echo "  ref:      $REF"
  echo "  checkout: $SRC"
  exit 0
fi

if [ "$MODE" = revert ]; then
  [ -f "$STOCK" ] || die "no stock backup at $STOCK; nothing to revert"
  cp -p "$STOCK" "$BIN.new" && mv -f "$BIN.new" "$BIN"
  rm -f "$MARKER"
  echo "reverted to stock: $(sha256_file "$BIN")"
  echo "restart vibe (or /exit + vibe --continue) to use it"
  exit 0
fi

# --- patch ---
if [ ! -d "$SRC/.git" ]; then
  echo "cloning mingf-ai/mistral-vibe -> $SRC"
  git clone --quiet --branch "$REF" https://github.com/mingf-ai/mistral-vibe "$SRC"
fi
git -C "$SRC" fetch origin --quiet
if git -C "$SRC" rev-parse --verify -q "refs/remotes/origin/$REF" >/dev/null; then
  git -C "$SRC" checkout -q -B "$REF" "refs/remotes/origin/$REF"
else
  git -C "$SRC" checkout -q "$REF"
fi
echo "building $REF (cargo build --release; first build takes a few minutes)…"
(cd "$SRC/vibe/cli-rust" && cargo build --release --quiet)
NEW="$SRC/vibe/cli-rust/target/release/vibe-rs"
[ -f "$NEW" ] || die "build produced no binary at $NEW"
NEW_SHA="$(sha256_file "$NEW")"
CUR_SHA="$(sha256_file "$BIN" 2>/dev/null || echo missing)"
if [ "$NEW_SHA" = "$CUR_SHA" ] && [ "$NEW_SHA" = "$(cat "$MARKER" 2>/dev/null || echo none)" ]; then
  echo "already patched at sha $NEW_SHA — nothing to do"
  exit 0
fi

# Adopt a stock backup left by a manual patch run, if any.
if [ ! -f "$STOCK" ]; then
  # shellcheck disable=SC2012
  # shellcheck disable=SC2012
  first="$(ls "$BINDIR"/vibe-rs.stock* 2>/dev/null | head -1 || true)"
  if [ -n "$first" ]; then
    mv "$first" "$STOCK"
    echo "adopted existing backup: $(basename "$first") -> vibe-rs.stock"
  fi
fi
if [ ! -f "$STOCK" ]; then
  echo "WARNING: no stock backup yet; backing up the CURRENT binary — verify it is stock." >&2
  cp -p "$BIN" "$STOCK"
fi

cp "$NEW" "$BIN.new" && mv -f "$BIN.new" "$BIN"
echo "$NEW_SHA" > "$MARKER"
if "$BIN" --version >/dev/null 2>&1; then
  echo "sanity: patched binary starts (--version ok)"
else
  echo "WARN: patched binary exited non-zero on --version; inspect, or revert: $0 --revert" >&2
fi
echo "patched: $NEW_SHA"
echo "restart vibe (or /exit + vibe --continue) to run it"
