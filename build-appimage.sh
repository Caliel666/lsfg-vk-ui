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

echo -e "${GREEN}Building ${APP_NAME} AppImage v${APP_VERSION} (Manual Build)...${NC}"
echo -e "${BLUE}Made by Cali666 • 2025${NC}"
echo ""

# --- 1. Cleanup ---
echo -e "${YELLOW}Cleaning up previous builds...${NC}"
rm -rf AppDir *.AppImage appimagetool-x86_64.AppImage

# --- 2. Build Rust App ---
echo -e "${YELLOW}Building Rust application in release mode...${NC}"
cargo build --release

# --- 3. Prepare AppDir Structure ---
echo -e "${YELLOW}Setting up AppDir structure...${NC}"
APPDIR="AppDir"
mkdir -p "${APPDIR}/usr/bin"
mkdir -p "${APPDIR}/usr/lib"
mkdir -p "${APPDIR}/usr/share/applications"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps" # This line is correct for hicolor
mkdir -p "${APPDIR}/usr/share/icons/scalable/apps"         # ADD THIS LINE for the direct scalable path
mkdir -p "${APPDIR}/usr/share/metainfo"
mkdir -p "${APPDIR}/usr/share/themes"
mkdir -p "${APPDIR}/usr/share/glib-2.0/schemas" # For GSettings schemas
mkdir -p "${APPDIR}/usr/share/locale" # For translations

# Copy binary
cp "target/release/${APP_NAME}" "${APPDIR}/usr/bin/"

# Copy desktop file and icon
cp "resources/${APP_ID}.desktop" "${APPDIR}/usr/share/applications/"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/${APP_ID}.png" # This copy is for hicolor/scalable
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/scalable/apps/${APP_ID}.png"         # This copy needs the new mkdir -p line
cp "resources/icons/lsfg-vk.png" "${APPDIR}/.DirIcon" # AppImage thumbnail icon

# --- 3.1 Copy System Icons ---
echo -e "${YELLOW}Copying system icons...${NC}"
# Copy only necessary icon themes. Adwaita is common for GTK/Adwaita apps.
cp -r /usr/share/icons/Adwaita "${APPDIR}/usr/share/icons/"
cp -r /usr/share/icons/hicolor "${APPDIR}/usr/share/icons/"
cp -r /usr/share/icons/Adwaita-symbolic "${APPDIR}/usr/share/icons/" || echo -e "${YELLOW}Warning: Adwaita-symbolic icons not found, continuing without them${NC}"

# --- 3.2 Create Metainfo File ---
echo -e "${YELLOW}Creating metainfo file...${NC}"
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

# --- 4. Manually Copy Libraries (including libadwaita) ---
echo -e "${YELLOW}Manually copying required libraries...${NC}"

# Function to copy a library and its direct dependencies
copy_library_with_deps() {
    local lib_path="$1"
    local dest_dir="$2"
    
    # Copy the library itself
    if [ -f "$lib_path" ]; then
        cp "$lib_path" "$dest_dir"
        echo "  Copied: $(basename "$lib_path")"
        
        # Get its direct dependencies
        # Filter out common system libraries that are expected to be present
        # and explicitly exclude gdk-pixbuf related libraries.
        ldd "$lib_path" | grep '=>' | grep -v 'libgdk_pixbuf' | grep -v 'libgdk-' | grep -v 'libgtk-' | awk '{print $3}' | while read -r dep_lib; do
            if [[ "$dep_lib" == /* ]] && [ -f "$dep_lib" ]; then
                # Only copy if it's not already in the AppDir/usr/lib
                if [ ! -f "${dest_dir}/$(basename "$dep_lib")" ]; then
                    copy_library_with_deps "$dep_lib" "$dest_dir"
                fi
            fi
        done
    else
        echo -e "${RED}Warning: Library not found - $lib_path${NC}"
    fi
}

# Find your Rust executable's direct dependencies
echo -e "${BLUE}  Copying Rust app dependencies:${NC}"
ldd "target/release/${APP_NAME}" | grep '=>' | awk '{print $3}' | while read -r lib; do
    if [[ "$lib" == /* ]]; then
        copy_library_with_deps "$lib" "${APPDIR}/usr/lib"
    fi
done

# Copy essential GTK/GLib/Adwaita libraries and their dependencies
echo -e "${BLUE}  Copying GTK/GLib/Adwaita and their dependencies:${NC}"
# List of core libraries you expect your GTK/Adwaita app to need
REQUIRED_LIBS=(
    "/usr/lib/x86_64-linux-gnu/libadwaita-1.so"
    "/usr/lib/x86_64-linux-gnu/libgtk-4.so"
    "/usr/lib/x86_64-linux-gnu/libgdk-4.so"
    "/usr/lib/x86_64-linux-gnu/libgio-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libglib-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libcairo.so"
    "/usr/lib/x86_64-linux-gnu/libpango-1.0.so"
    "/usr/lib/x86_64-linux-gnu/libharfbuzz.so"
    "/usr/lib/x86_64-linux-gnu/libfontconfig.so"
    "/usr/lib/x86_64-linux-gnu/libfreetype.so"
    "/usr/lib/x86_64-linux-gnu/libgobject-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libepoxy.so" # Often a dependency for GTK4
    "/usr/lib/x86_64-linux-gnu/libgmodule-2.0.so" # For GModule
)

for lib in "${REQUIRED_LIBS[@]}"; do
    if [ -f "$lib" ]; then
        copy_library_with_deps "$lib" "${APPDIR}/usr/lib"
    else
        echo -e "${RED}Warning: Essential library not found - $lib. This might cause issues.${NC}"
    fi
done

# --- 4.1 Copy GSettings Schemas ---
echo -e "${YELLOW}Copying GSettings schemas...${NC}"
# Copy GLib and Adwaita schemas
# These are crucial for GTK/Adwaita applications to function correctly
cp /usr/share/glib-2.0/schemas/gschemas.compiled "${APPDIR}/usr/share/glib-2.0/schemas/" || true
# Copy individual schema files that might be relevant if gschemas.compiled isn't enough
# It's safer to copy the compiled one if available and ensure it's up-to-date.
# If you find runtime errors about missing schemas, you might need to copy more specific XML files
# and run `glib-compile-schemas` inside the AppDir, but for now, we'll rely on the compiled one.
# For example, if you see issues, you might need:
# cp /usr/share/glib-2.0/schemas/org.gnome.adwaita.gresource.xml "${APPDIR}/usr/share/glib-2.0/schemas/"
# cp /usr/share/glib-2.0/schemas/org.gtk.Settings.FileChooser.gschema.xml "${APPDIR}/usr/share/glib-2.0/schemas/"
# cp /usr/share/glib-2.0/schemas/org.gnome.desktop.interface.gschema.xml "${APPDIR}/usr/share/glib-2.0/schemas/"

# --- 4.2 Copy GDK-Pixbuf Loaders (selectively, to avoid full pixbuf) ---
# This is where it gets tricky. If your app relies on ANY image loading (PNG, JPEG),
# you might need *some* GDK-Pixbuf functionality, but not the entire module system.
# The goal is to avoid the full plugin infrastructure.
echo -e "${YELLOW}Copying minimal GDK-Pixbuf components (if absolutely necessary, otherwise skip)...${NC}"
# Generally, modern GTK apps use Cairo for many drawing operations.
# You might not need GDK-Pixbuf directly unless you explicitly load images with it.
# If your app has issues with image display, consider adding specific image loaders,
# e.g., /usr/lib/x86_64-linux-gnu/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader-png.so
# and adjust your AppRun to point to this isolated loader if needed.
# For now, we'll try to omit them as per your request to avoid "gdk things like pixbuff".

# --- 5. Create AppRun Script ---
echo -e "${YELLOW}Creating custom AppRun script...${NC}"
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash

# Define the AppDir root
HERE="$(dirname "$(readlink -f "${0}")")"

# Set up library path to include our bundled libraries first
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"

# Set up XDG data directories for icons, applications, and metainfo
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"

# Set GTK data prefix for themes and modules
export GTK_DATA_PREFIX="${HERE}/usr"

# Set up GSettings schema directory
export GSETTINGS_SCHEMA_DIR="${HERE}/usr/share/glib-2.0/schemas:${GSETTINGS_SCHEMA_DIR}"

# Explicitly unset GDK_PIXBUF environment variables to prevent loading from system
unset GDK_PIXBUF_MODULE_FILE
unset GDK_PIXBUF_MODULEDIR

# Execute the main application binary
exec "${HERE}/usr/bin/lsfg-vk-ui" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

# --- 6. Download appimagetool ---
echo -e "${YELLOW}Downloading appimagetool...${NC}"
wget -qc "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
chmod +x appimagetool-x86_64.AppImage

# --- 7. Generate AppImage ---
echo -e "${YELLOW}Generating AppImage using appimagetool...${NC}"
ARCH=x86_64 ./appimagetool-x86_64.AppImage "${APPDIR}" "${FINAL_APPIMAGE_NAME}"

# --- 8. Final Cleanup ---
echo -e "${YELLOW}Cleaning up build directories...${NC}"
rm -rf AppDir appimagetool-x86_64.AppImage

# --- Success Message ---
echo ""
echo -e "${GREEN}✓ AppImage built successfully: ${FINAL_APPIMAGE_NAME}${NC}"
echo -e "${GREEN}✓ File size: $(du -h "${FINAL_APPIMAGE_NAME}" | cut -f1)${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  chmod +x ${FINAL_APPIMAGE_NAME}"
echo "  ./${FINAL_APPIMAGE_NAME}"