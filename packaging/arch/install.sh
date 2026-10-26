#!/usr/bin/env bash
# Build Serein from source on Arch/CachyOS/Manjaro/EndeavourOS and install it with pacman.
# Run from a checkout:   git clone https://github.com/HaraldErik/Serein-Fork && cd Serein-Fork && bash packaging/arch/install.sh
# Or in one line:        curl -fsSL https://raw.githubusercontent.com/HaraldErik/Serein-Fork/main/packaging/arch/install.sh | bash
# Run as your normal user; sudo is only used for pacman. Nothing is installed outside pacman and ~/.rustup, ~/.cargo.
set -euo pipefail

REPO_URL="${SEREIN_REPO_URL:-https://github.com/HaraldErik/Serein-Fork}"
REPO_REF="${SEREIN_REF:-main}"

if [ "$(id -u)" -eq 0 ]; then
  echo "Run as a normal user, not root (makepkg refuses root). sudo is used where needed." >&2
  exit 1
fi
if ! command -v pacman >/dev/null; then
  echo "This script needs pacman (Arch-based distribution)." >&2
  exit 1
fi

# Use the current checkout when run from inside one; otherwise clone into a temp directory.
root=""
if script_dir=$(cd "$(dirname "${BASH_SOURCE[0]:-}")" 2>/dev/null && pwd) && [ -f "$script_dir/../../Cargo.toml" ]; then
  root=$(cd "$script_dir/../.." && pwd)
else
  root=$(mktemp -d)
  trap 'rm -rf "$root"' EXIT
  git clone --depth 1 --branch "$REPO_REF" "$REPO_URL" "$root" 2>/dev/null \
    || { sudo pacman -S --needed --noconfirm git && git clone --depth 1 --branch "$REPO_REF" "$REPO_URL" "$root"; }
fi
cd "$root"

echo ">> Installing build dependencies"
sudo pacman -S --needed --noconfirm base-devel cmake pkgconf curl ca-certificates git tar gzip xz \
  python coreutils desktop-file-utils file gtk4 webkitgtk-6.0 fontconfig libxkbcommon \
  libxkbcommon-x11 wayland libx11 libxi libxrandr libxcursor vulkan-headers vulkan-icd-loader \
  alsa-lib libpulse gstreamer gst-plugins-base-libs gst-plugins-good libarchive

# rustup and the `rust` package conflict on Arch: keep whichever is already installed.
if ! command -v rustup >/dev/null && ! command -v cargo >/dev/null; then
  sudo pacman -S --needed --noconfirm rustup
fi

echo ">> Preparing the pinned Rust toolchain"
if command -v rustup >/dev/null; then
  rustup show active-toolchain >/dev/null 2>&1 || rustup toolchain install --profile minimal
else
  echo "Using system Rust ($(rustc --version)); the repo pins $(sed -n 's/^channel = \"\(.*\)\"/\1/p' rust-toolchain.toml) and only rustup can fetch it."
fi

echo ">> Building (this takes a while)"
cargo xtask package --format arch

package=$(ls -t dist/*.pkg.tar.* 2>/dev/null | grep -v '\.sig$' | head -n1 || true)
if [ -z "$package" ]; then
  echo "Build finished but no package was found in dist/." >&2
  exit 1
fi
echo ">> Installing $package with pacman"
sudo pacman -U --noconfirm "$package"
echo ">> Refreshing desktop entries, icons and the KDE application menu"
sudo update-desktop-database /usr/share/applications 2>/dev/null || true
sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true
for rebuild in kbuildsycoca6 kbuildsycoca5; do
  command -v "$rebuild" >/dev/null && { "$rebuild" --noincremental >/dev/null 2>&1 || true; break; }
done
echo "Done. Start Serein from your app menu or run: serein"
