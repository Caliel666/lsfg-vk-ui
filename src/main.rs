#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# --- Configuration ---
APP_NAME="lsfg-vk-ui"
APP_ID="com.cali666.lsfg-vk-ui"
APP_VERSION=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

if [ -z "$APP_VERSION" ]; then
    echo -e "${RED}Error: Could not determine app version from Cargo.toml.${NC}"
    exit 1
fi

FINAL_APPIMAGE_NAME="${APP_NAME}-${APP_VERSION}-x86_64.AppImage"

echo -e "${GREEN}Building ${APP_NAME} AppImage v${APP_VERSION}...${NC}"
echo -e "${BLUE}Made by Cali666 • 2025${NC}"
echo ""

# --- 1. Cleanup ---
echo -e "${YELLOW}Cleaning up previous builds...${NC}"
rm -rf AppDir *.AppImage linuxdeploy*.AppImage linuxdeploy-plugin-gtk.sh

# --- 2. Build Rust App ---
echo -e "${YELLOW}Building Rust application in release mode...${NC}"
cargo build --release

# --- 3. Prepare AppDir ---
echo -e "${YELLOW}Setting up AppDir structure...${NC}"
APPDIR="AppDir"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/metainfo"

# Copy binary
cp "target/release/${APP_NAME}" "${APPDIR}/usr/bin/"

# Copy desktop file and icon
cp "resources/${APP_ID}.desktop" "${APPDIR}/usr/share/applications/"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"

# Create metainfo file (same as before)
cat > "${APPDIR}/usr/share/metainfo/${APP_ID}.metainfo.xml" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>${APP_ID}</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>MIT</project_license>
  <name>LSFG-VK UI</name>
  <summary>Lossless Scaling Frame Generation Configuration Tool</summary>
  <description>
    <p>
      A configuration tool for Lossless Scaling Frame Generation, allowing you to create and manage game profiles with custom settings.
    </p>
  </description>
  <developer_name>Cali666</developer_name>
  <url type="homepage">https://github.com/PancakeTAS/lsfg-vk</url>
  <categories>
    <category>Game</category>
    <category>Utility</category>
  </categories>
  <releases>
    <release version="${APP_VERSION}" date="$(date +%Y-%m-%d)">
      <description>
        <p>New release.</p>
      </description>
    </release>
  </releases>
</component>
EOF

# --- 4. Download Deployment Tools ---
echo -e "${YELLOW}Downloading linuxdeploy and plugins...${NC}"
wget -qc "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
wget -qc "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
chmod +x linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh

# --- 5. Patch GTK Plugin ---
echo -e "${YELLOW}Patching GTK plugin...${NC}"
# Disable theme bundling and icon bundling
sed -i 's|export GTK_THEME="\$APPIMAGE_GTK_THEME"|# &|' linuxdeploy-plugin-gtk.sh

# --- 6. Create wrapper script to set up environment ---
echo -e "${YELLOW}Creating wrapper script...${NC}"
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash

# Set up environment for system icons
export XDG_DATA_DIRS="${APPDIR}/usr/share:${XDG_DATA_DIRS}:/usr/local/share:/usr/share"
export GTK_THEME=Adwaita
export ICON_THEME=Adwaita

# Find the executable
HERE="$(dirname "$(readlink -f "${0}")")"
exec "$HERE/usr/bin/lsfg-vk-ui" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# --- 7. Run linuxdeploy ---
echo -e "${YELLOW}Bundling dependencies and creating AppImage...${NC}"

# Tell linuxdeploy not to bundle icons or themes
export NO_BUNDLE_ICONS=1
export NO_BUNDLE_THEMES=1

# Run linuxdeploy
LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/usr/lib \
NO_STRIP=1 ./linuxdeploy-x86_64.AppImage \
    --appdir "${APPDIR}" \
    --plugin gtk \
    --output appimage

GENERATED_APPIMAGE=$(find . -maxdepth 1 -name "*.AppImage" ! -name "linuxdeploy-x86_64.AppImage" -print -quit)
mv "${GENERATED_APPIMAGE}" "${FINAL_APPIMAGE_NAME}"

# --- 8. Final Cleanup ---
echo -e "${YELLOW}Cleaning up build directories...${NC}"
rm -rf AppDir linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh

# --- Success Message ---
echo ""
echo -e "${GREEN}✓ AppImage built successfully: ${FINAL_APPIMAGE_NAME}${NC}"
echo -e "${GREEN}✓ File size: $(du -h "${FINAL_APPIMAGE_NAME}" | cut -f1)${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  chmod +x ${FINAL_APPIMAGE_NAME}"
echo "  ./${FINAL_APPIMAGE_NAME}"