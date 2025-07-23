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
mkdir -p "${APPDIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${APPDIR}/usr/share/icons/scalable/apps"
mkdir -p "${APPDIR}/usr/share/metainfo"
mkdir -p "${APPDIR}/usr/share/themes"
mkdir -p "${APPDIR}/usr/share/glib-2.0/schemas" # For GSettings schemas
mkdir -p "${APPDIR}/usr/share/locale" # For translations

# Copy binary
cp "target/release/${APP_NAME}" "${APPDIR}/usr/bin/"

# Copy desktop file and icon
cp "resources/${APP_ID}.desktop" "${APPDIR}/usr/share/applications/"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/scalable/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/.DirIcon" # AppImage thumbnail icon

# --- 3.1 Copy System Icons ---
echo -e "${YELLOW}Copying system icons...${NC}"
cp -r /usr/share/icons/Adwaita "${APPDIR}/usr/share/icons/" || true
cp -r /usr/share/icons/hicolor "${APPDIR}/usr/share/icons/" || true
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

# Global array to keep track of copied libraries to avoid redundant checks/copies
declare -A COPIED_LIBS

# Function to copy a library and its direct dependencies
copy_library_with_deps() {
    local lib_path="$1"
    local dest_dir="$2"
    local lib_basename=$(basename "$lib_path")

    # If already copied, skip
    if [[ -n "${COPIED_LIBS[$lib_basename]}" ]]; then
        return 0
    fi
    
    # Check if the library path is a symbolic link and resolve it to its real path
    if [ -L "$lib_path" ]; then
        lib_path=$(readlink -f "$lib_path")
        lib_basename=$(basename "$lib_path") # Update basename after resolving symlink
    fi

    # Check if the library file actually exists before attempting to copy
    if [ ! -f "$lib_path" ]; then
        echo -e "${YELLOW}Warning: Library file not found - $lib_path${NC}"
        return 1 # Indicate failure so the caller can handle it (e.g., || true)
    fi

    # Explicitly exclude common system libraries that should not be bundled
    case "$lib_basename" in
        # Basic C/Unix/Kernel libraries - almost always provided by the OS
        libc.so*|libm.so*|libpthread.so*|libdl.so*|librt.so*|libutil.so*|libnsl.so*|libnss_*.so*|libresolv.so*|libanl.so*|ld-linux-x86-64.so*|\
        # Desktop environment/system specifics (e.g., Wayland, X11, D-Bus, etc.)
        libwayland-*.so*|libX*.so*|libxcb*.so*|libdbus*.so*|libudev.so*|libsystemd.so*|libappindicator*.so*|libayatana-appindicator*.so*|libdbus-*.so*|\
        # Common compression/archive/crypto libs often system provided
        libz.so*|liblzma.so*|libbz2.so*|libgcrypt.so*|libgpg-error.so*|libcrypto.so*|libssl.so*|\
        # GCC runtime libs
        libgcc_s.so*|libstdc++.so*|\
        # GDK-Pixbuf and problematic GTK/GDK components
        libgdk_pixbuf*.so*|libgdk-pixbuf*.so*|*gdk-pixbuf*|\
        # Some general purpose ones that are almost always system-provided
        libcap.so*|libattr.so*|libacl.so*|libselinux.so*|libtirpc.so*|libmount.so*|libuuid.so*|libblkid.so*|libffi.so*|libkeyutils.so*|libaudit.so*|libjson-c.so*|libxml2.so*|libxslt.so*|libexpat.so*|libelf.so*|libunwind.so*|libdw.so*|libnuma.so*|libjemalloc.so*|libtcmalloc.so*|libm.so*)
            echo "  Skipping common system lib: $lib_basename"
            COPIED_LIBS[$lib_basename]=1 # Mark as "handled" to avoid re-processing
            return 0
            ;;
        *)
            # Continue with copy
            ;;
    esac

    # Copy the library to the AppDir/usr/lib
    cp "$lib_path" "${dest_dir}/$lib_basename"
    echo "  Copied: $lib_basename"
    COPIED_LIBS[$lib_basename]=1 # Mark as copied

    # Get its direct dependencies recursively
    # Use 'grep -oP' with lookbehind to extract only the path, avoid other parts of ldd output
    # Exclude common system libraries and gdk-pixbuf directly from ldd output parsing
    ldd "$lib_path" 2>/dev/null | grep -oP '=> \K(/[^[:space:]]+)' | grep -v 'libgdk_pixbuf' | grep -v 'libgdk-pixbuf' | while read -r dep_lib; do
        copy_library_with_deps "$dep_lib" "$dest_dir" || true # Continue even if dep is ignored/missing
    done
}

# --- IMPORTANT: Carefully select your initial REQUIRED_LIBS ---
# Start with your application's binary, then core GTK4/Adwaita libraries.
# Only include libraries here that you know are ABSOLUTELY necessary and
# which you want to be the starting point for dependency resolution.
echo -e "${BLUE}  Copying core application and GTK/GLib/Adwaita dependencies:${NC}"
REQUIRED_LIBS=(
    "target/release/${APP_NAME}" # Start with your main executable
    "/usr/lib/x86_64-linux-gnu/libadwaita-1.so"
    "/usr/lib/x86_64-linux-gnu/libgtk-4.so"
    # Note: libgdk-4.so and others like libgio, libglib, libcairo, etc.,
    # will be pulled as dependencies of libgtk-4.so and libadwaita-1.so.
    # If your app requires any *other* specific libraries not covered by this chain, add them here.
)

# Process the explicitly required libraries
for lib in "${REQUIRED_LIBS[@]}"; do
    # For the main executable, we copy it to usr/bin, not usr/lib
    if [ "$lib" = "target/release/${APP_NAME}" ]; then
        # Dependencies of the main executable will still go to usr/lib
        ldd "$lib" 2>/dev/null | grep -oP '=> \K(/[^[:space:]]+)' | grep -v 'libgdk_pixbuf' | grep -v 'libgdk-pixbuf' | while read -r dep_lib; do
            copy_library_with_deps "$dep_lib" "${APPDIR}/usr/lib" || true
        done
    else
        copy_library_with_deps "$lib" "${APPDIR}/usr/lib" || true
    fi
done

# --- 4.1 Copy GSettings Schemas ---
echo -e "${YELLOW}Copying GSettings schemas...${NC}"
# Copy GLib and Adwaita schemas
cp /usr/share/glib-2.0/schemas/gschemas.compiled "${APPDIR}/usr/share/glib-2.0/schemas/" || true

# --- 5. Create AppRun Script ---
echo -e "${YELLOW}Creating custom AppRun script...${NC}"
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash

HERE="$(dirname "$(readlink -f "${0}")")"

export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export GTK_DATA_PREFIX="${HERE}/usr"
export GSETTINGS_SCHEMA_DIR="${HERE}/usr/share/glib-2.0/schemas:${GSETTINGS_SCHEMA_DIR}"

# Explicitly unset GDK_PIXBUF environment variables
unset GDK_PIXBUF_MODULE_FILE
unset GDK_PIXBUF_MODULEDIR

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