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
# Dynamically get version from Cargo.toml to name the output file
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

# --- Bundle symbolic icons to ensure they are available ---
echo -e "${YELLOW}Bundling required symbolic icons...${NC}"
ICON_DEST_DIR="${APPDIR}/usr/share/icons/hicolor/scalable/actions"
mkdir -p "${ICON_DEST_DIR}"

# List of icons to bundle. Using standard Adwaita/GNOME icons ensures they
# are found on the build runner and provides a consistent look.
ICONS_TO_BUNDLE=(
    "org.gnome.Settings-symbolic.svg"
    "document-edit-symbolic.svg"
    "edit-delete-symbolic.svg"
)

for icon_name in "${ICONS_TO_BUNDLE[@]}"; do
    icon_path=$(find /usr/share/icons -name "${icon_name}" -print -quit)
    if [ -n "$icon_path" ]; then
        echo "Found icon to bundle: ${icon_path} -> ${ICON_DEST_DIR}/"
        cp "${icon_path}" "${ICON_DEST_DIR}/"
    else
        echo -e "${RED}Error: Could not find required icon '${icon_name}' on the build system.${NC}"
        echo -e "${RED}This might mean the 'adwaita-icon-theme' package is missing or incomplete.${NC}"
        exit 1
    fi
done

# --- 4. Update Icon Cache ---
echo -e "${YELLOW}Updating icon cache to make bundled icons discoverable...${NC}"
gtk-update-icon-cache -f -t "${APPDIR}/usr/share/icons/hicolor"

# Create a dynamic metainfo file
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

# --- 5. Download Deployment Tools ---
echo -e "${YELLOW}Downloading linuxdeploy and plugins...${NC}"
wget -qc "https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
# Per your request, using the raw file from the master branch for the GTK plugin.
wget -qc "https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh"
chmod +x linuxdeploy-x86_64.AppImage linuxdeploy-plugin-gtk.sh

# --- 6. Patch GTK Plugin ---
echo -e "${YELLOW}Patching GTK plugin to use libadwaita's default theme...${NC}"
# By commenting out the line that sets GTK_THEME in the plugin's generated hook,
# we allow libadwaita to use its own built-in theme. This correctly handles
# light/dark modes and avoids visual glitches from bundling incomplete system themes.
sed -i 's|export GTK_THEME="\$APPIMAGE_GTK_THEME"|# &|' linuxdeploy-plugin-gtk.sh

# --- 7. Run linuxdeploy to Bundle Dependencies ---
echo -e "${YELLOW}Bundling dependencies and creating AppImage...${NC}"

# Run linuxdeploy. It will find the desktop file, icon, and executable.
# The GTK plugin will automatically find and bundle libadwaita and other GTK-specific files.
# By setting NO_STRIP=1, we prevent linuxdeploy from using its internal `strip` command,
# which can be too old to understand modern ELF sections like .relr.dyn on newer systems.
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
