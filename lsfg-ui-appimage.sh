#!/bin/sh

set -eux

ARCH=$(uname -m)
URUNTIME="https://github.com/VHSgunzo/uruntime/releases/latest/download/uruntime-appimage-dwarfs-$ARCH"
URUNTIME_LITE="https://github.com/VHSgunzo/uruntime/releases/latest/download/uruntime-appimage-dwarfs-lite-$ARCH"
UPINFO="gh-releases-zsync|$(echo $GITHUB_REPOSITORY | tr '/' '|')|latest|*$ARCH.AppImage.zsync"
SHARUN="https://github.com/VHSgunzo/sharun/releases/latest/download/sharun-$ARCH-aio"

VERSION=$(awk -F'=|"' '/^version/{print $3}' ./lsfg-ui/Cargo.toml)
echo "$VERSION" > ~/version

# deploy dependencies
mkdir -p ./AppDir/shared/bin
cp -v ./resources/*.desktop         ./AppDir
cp -v ./resources/icons/lsfg-vk.png ./AppDir/lsfg-ui.png
cp -v ./resources/icons/lsfg-vk.png ./AppDir/.DirIcon
mv -v ./target/release/lsfg-vk-ui   ./AppDir/shared/bin && (
	cd ./AppDir
	wget --retry-connrefused --tries=30 "$SHARUN" -O ./sharun-aio
	chmod +x ./sharun-aio
	xvfb-run -a ./sharun-aio l -p -v -e -s -k  \
		./shared/bin/lsfg-vk-ui            \
		/usr/lib/gdk-pixbuf-*/*/loaders/*  \
		/usr/lib/gio/modules/libdconfsettings.so
	rm -f ./sharun-aio
	ln ./sharun ./AppRun
	./sharun -g
)

# turn AppDIr into appimage with uruntime
wget --retry-connrefused --tries=30 "$URUNTIME"      -O  ./uruntime
wget --retry-connrefused --tries=30 "$URUNTIME_LITE" -O  ./uruntime-lite
chmod +x ./uruntime*

# Add udpate info to runtime
echo "Adding update information \"$UPINFO\" to runtime..."
./uruntime-lite --appimage-addupdinfo "$UPINFO"

echo "Generating AppImage..."
./uruntime --appimage-mkdwarfs -f \
	--set-owner 0 --set-group 0 \
	--no-history --no-create-timestamp \
	--compression zstd:level=22 -S26 -B8 \
	--header uruntime-lite \
	-i ./AppDir -o ./lsfg-ui-"$VERSION"-anylinux-"$ARCH".AppImage

echo "Generating zsync file..."
zsyncmake ./*.AppImage -u ./*.AppImage

mkdir -p ./dist
mv -v ./*.AppImage* ./dist

echo "All Done!"
