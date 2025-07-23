#!/bin/sh

set -ex
ARCH="$(uname -m)"

echo "Installing build dependencies..."
echo "---------------------------------------------------------------"
pacman -Syu --noconfirm    \
	base-devel         \
	cargo              \
	curl               \
	git                \
	gtk4               \
	libadwaita         \
	libxtst            \
	pipewire-audio     \
	pulseaudio         \
	wget               \
	xorg-server-xvfb   \
	zsync

case "$ARCH" in
	'x86_64')
		PKG_TYPE='x86_64.pkg.tar.zst'
		;;
	'aarch64')
		PKG_TYPE='aarch64.pkg.tar.xz'
		;;
	''|*)
		echo "Unknown cpu arch: $ARCH"
		exit 1
		;;
esac

LIBXML_URL="https://github.com/pkgforge-dev/llvm-libs-debloated/releases/download/continuous/libxml2-iculess-$PKG_TYPE"
MESA_URL="https://github.com/pkgforge-dev/llvm-libs-debloated/releases/download/continuous/mesa-mini-$PKG_TYPE"
LLVM_URL="https://github.com/pkgforge-dev/llvm-libs-debloated/releases/download/continuous/llvm-libs-nano-$PKG_TYPE"

# We need to build here because the rust compiler hates the smaller llvm
cargo build --release

echo "Installing debloated pckages..."
echo "---------------------------------------------------------------"
wget --retry-connrefused --tries=30 "$LIBXML_URL" -O  ./libxml2.pkg.tar.zst
wget --retry-connrefused --tries=30 "$LLVM_URL"   -O  ./llvm-libs.pkg.tar.zst
wget --retry-connrefused --tries=30 "$MESA_URL"   -O  ./mesa.pkg.tar.zst

pacman -U --noconfirm ./*.pkg.tar.zst
rm -f ./*.pkg.tar.zst

echo "All done!"
echo "---------------------------------------------------------------"
