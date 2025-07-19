#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building LSFG-VK UI AppImage...${NC}"
echo -e "${BLUE}Made by Cali666 • 2025${NC}"
echo ""

# Clean up any existing build artifacts
echo -e "${YELLOW}Cleaning up previous builds...${NC}"
rm -rf AppDir *.AppImage appimagetool-x86_64.AppImage

# Build the application in release mode
echo -e "${YELLOW}Building Rust application...${NC}"
cargo build --release

# Create AppDir structure
echo -e "${YELLOW}Setting up AppDir structure...${NC}"
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/icons/hicolor/256x256/apps
mkdir -p AppDir/usr/share/metainfo

# Copy binary
cp target/release/lsfg-vk-ui AppDir/usr/bin/

# Copy desktop file
cp resources/com.cali666.lsfg-vk-ui.desktop AppDir/usr/share/applications/

# Copy icon
cp resources/icons/lsfg-vk.png AppDir/usr/share/icons/hicolor/256x256/apps/com.cali666.lsfg-vk-ui.png

# Create AppRun script
cat > AppDir/AppRun << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export PATH="${HERE}/usr/bin/:${PATH}"
exec "${HERE}/usr/bin/lsfg-vk-ui" "$@"
EOF
chmod +x AppDir/AppRun

# Copy desktop file and icon to AppDir root (required for AppImage)
cp resources/com.cali666.lsfg-vk-ui.desktop AppDir/
cp resources/icons/lsfg-vk.png AppDir/com.cali666.lsfg-vk-ui.png

# Create metainfo file
cat > AppDir/usr/share/metainfo/com.cali666.lsfg-vk-ui.metainfo.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>com.cali666.lsfg-vk-ui</id>
  <metadata_license>CC0-1.0</metadata_license>
  <project_license>MIT</project_license>
  <name>LSFG-VK UI</name>
  <summary>Lossless Scaling Frame Generation Configuration Tool</summary>
  <description>
    <p>
      LSFG-VK UI is a configuration tool for Lossless Scaling Frame Generation,
      allowing you to create and manage game profiles with custom settings for
      frame generation, HDR mode, and performance optimization.
    </p>
  </description>
  <developer_name>Cali666</developer_name>
  <url type="homepage">https://github.com/PancakeTAS/lsfg-vk</url>
  <categories>
    <category>Game</category>
    <category>Settings</category>
  </categories>
  <keywords>
    <keyword>lossless</keyword>
    <keyword>scaling</keyword>
    <keyword>frame</keyword>
    <keyword>generation</keyword>
    <keyword>gaming</keyword>
  </keywords>
  <releases>
    <release version="0.1.0" date="2025-01-20">
      <description>
        <p>Initial release of LSFG-VK UI</p>
      </description>
    </release>
  </releases>
</component>
EOF

# Download appimagetool if it doesn't exist
if [ ! -f "appimagetool-x86_64.AppImage" ]; then
    echo -e "${YELLOW}Downloading appimagetool...${NC}"
    wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x appimagetool-x86_64.AppImage
fi

# Build AppImage
echo -e "${YELLOW}Creating AppImage...${NC}"
./appimagetool-x86_64.AppImage AppDir LSFG-VK_UI-x86_64.AppImage

# Clean up build directories but keep the AppImage
echo -e "${YELLOW}Cleaning up build directories...${NC}"
rm -rf AppDir appimagetool-x86_64.AppImage

echo ""
echo -e "${GREEN}✓ AppImage built successfully: LSFG-VK_UI-x86_64.AppImage${NC}"
echo -e "${GREEN}✓ File size: $(du -h LSFG-VK_UI-x86_64.AppImage | cut -f1)${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  • Run directly: ./LSFG-VK_UI-x86_64.AppImage"
echo "  • Make executable and run: chmod +x LSFG-VK_UI-x86_64.AppImage && ./LSFG-VK_UI-x86_64.AppImage"
echo ""
echo -e "${BLUE}For system integration:${NC}"
echo "  • Copy to ~/.local/bin/ and add to PATH"
echo "  • Copy the .desktop file to ~/.local/share/applications/"
echo "  • Copy the icon to ~/.local/share/icons/hicolor/256x256/apps/"
