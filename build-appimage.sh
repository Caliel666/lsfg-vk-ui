#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

echo -e "${YELLOW}Cleaning up previous builds...${NC}"
rm -rf AppDir *.AppImage appimagetool-x86_64.AppImage

echo -e "${YELLOW}Building Rust application in release mode...${NC}"
cargo build --release

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
mkdir -p "${APPDIR}/usr/share/glib-2.0/schemas"
mkdir -p "${APPDIR}/usr/share/locale"

cp "target/release/${APP_NAME}" "${APPDIR}/usr/bin/"

cp "resources/${APP_ID}.desktop" "${APPDIR}/usr/share/applications/"
cp "resources/${APP_ID}.desktop" "${APPDIR}"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/256x256/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/hicolor/scalable/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/usr/share/icons/scalable/apps/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/${APP_ID}.png"
cp "resources/icons/lsfg-vk.png" "${APPDIR}/.DirIcon"

echo -e "${YELLOW}Copying system icons...${NC}"
cp -r /usr/share/icons/Adwaita "${APPDIR}/usr/share/icons/" || true
cp -r /usr/share/icons/hicolor "${APPDIR}/usr/share/icons/" || true

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

LIB_PATH_PREFIX="/usr/lib/x86_64-linux-gnu"
if [ ! -d "${LIB_PATH_PREFIX}" ]; then
    echo -e "${YELLOW}Warning: ${LIB_PATH_PREFIX} not found. Falling back to /usr/lib/${NC}"
    LIB_PATH_PREFIX="/usr/lib"
fi

echo -e "${YELLOW}Manually copying required libraries...${NC}"

# Declare associative arrays globally to ensure persistence across subshells
declare -A COPIED_PATHS        # Tracks libraries that have been successfully copied to AppDir
declare -A ALL_DEPENDENCIES    # Tracks all unique, non-excluded dependencies identified for copying
declare -A PROCESSED_FOR_DEPS  # Tracks libraries whose direct dependencies have already been processed (ldd run)

# Helper function to get direct dependencies of a library, resolving paths and applying exclusions
get_library_deps() {
    lib_path="$1"
    resolved_lib_path=$(readlink -f "$lib_path")
    lib_basename=$(basename "$resolved_lib_path")
    deps=""

    # Define excluded patterns for system libraries
    excluded_patterns="
        libc.so.*|libm.so.*|libpthread.so.*|libdl.so.*|librt.so.*|libutil.so.*|
        libnsl.so.*|libnss_.*.so.*|libresolv.so.*|libanl.so.*|ld-linux-x86-64.so.*|
        libwayland-.*.so.*|libX.*.so.*|libxcb.*.so.*|libdbus.*.so.*|libudev.so.*|
        libsystemd.so.*|libappindicator.*.so.*|libayatana-appindicator.*.so.*|
        libdbus-.*.so.*|libz.so.*|liblzma.so.*|libbz2.so.*|libgcrypt.so.*|
        libgpg-error.so.*|libssl.so.*|libgcc_s.so.*|libstdc++.so.*|
        libgdk_pixbuf.*.so.*|libgdk-pixbuf.*.so.*|.*gdk-pixbuf.*|libcap.so.*|
        libattr.so.*|libacl.so.*|libselinux.so.*|libtirpc.so.*|libmount.so.*|
        libuuid.so.*|libblkid.so.*|libffi.so.*|libkeyutils.so.*|libaudit.so.*|
        libjson-c.so.*|libxml2.so.*|libxslt.so.*|libexpat.so.*|libelf.so.*|
        libunwind.so.*|libdw.so.*|libnuma.so.*|libjemalloc.so.*|libtcmalloc.so.*|
        libm.so.*
    "
    excluded_patterns=$(echo "$excluded_patterns" | tr -d '[:space:]')

    # Run ldd and filter dependencies
    ldd "$resolved_lib_path" 2>/dev/null | grep -oP '=> \K(/[^[:space:]]+)' | \
        while read -r dep_lib; do
            resolved_dep_lib=$(readlink -f "$dep_lib")
            dep_basename=$(basename "$resolved_dep_lib")

            if [[ "$dep_basename" =~ ^($excluded_patterns)$ ]]; then
                continue # Skip common system dependencies
            fi
            deps="$deps $resolved_dep_lib"
        done
    echo "$deps" # Return space-separated list of dependencies
}

echo -e "${BLUE}  Collecting all necessary application and GTK/GLib/Adwaita dependencies:${NC}"

# Initialize a queue for dependency processing (breadth-first search)
declare -a PENDING_QUEUE
# Add the main application executable and core GTK/GLib/Adwaita libs to the queue
REQUIRED_LIBS=(
    "target/release/${APP_NAME}"
    "${LIB_PATH_PREFIX}/libadwaita-1.so"
    "${LIB_PATH_PREFIX}/libgtk-4.so"
)

for lib in "${REQUIRED_LIBS[@]}"; do
    resolved_lib=$(readlink -f "$lib")
    if [[ ! -n "${PROCESSED_FOR_DEPS[$resolved_lib]}" ]]; then
        PENDING_QUEUE+=("$resolved_lib")
        PROCESSED_FOR_DEPS[$resolved_lib]=1
    fi
done

# Process dependencies iteratively until the queue is empty
while [ ${#PENDING_QUEUE[@]} -gt 0 ]; do
    current_lib="${PENDING_QUEUE[0]}"
    PENDING_QUEUE=("${PENDING_QUEUE[@]:1}") # Dequeue the first element

    # Add the current library to the list of all dependencies to be copied,
    # unless it's the main application executable (which is copied separately).
    if [[ "$current_lib" != "$(readlink -f target/release/${APP_NAME})" ]]; then
        ALL_DEPENDENCIES["$current_lib"]=1
    fi

    # Get direct dependencies of the current library
    new_deps=$(get_library_deps "$current_lib")

    # Add new, unprocessed dependencies to the queue
    for dep in $new_deps; do
        if [[ ! -n "${PROCESSED_FOR_DEPS[$dep]}" ]]; then
            PENDING_QUEUE+=("$dep")
            PROCESSED_FOR_DEPS[$dep]=1
        fi
    done
done

echo -e "${BLUE}  Copying collected unique dependencies to AppDir:${NC}"
# Now, iterate through all collected unique dependencies and copy them
for lib_path in "${!ALL_DEPENDENCIES[@]}"; do
    lib_basename=$(basename "$lib_path") # Removed 'local'
    
    # Only copy if it hasn't been copied yet (should be true for all in ALL_DEPENDENCIES, but good for robustness)
    if [[ -z "${COPIED_PATHS[$lib_path]}" ]]; then
        cp "$lib_path" "${APPDIR}/usr/lib/$lib_basename"
        echo "  Copied: $lib_basename"
        COPIED_PATHS[$lib_path]=1
    fi
done

echo -e "${YELLOW}Copying GSettings schemas...${NC}"
cp /usr/share/glib-2.0/schemas/gschemas.compiled "${APPDIR}/usr/share/glib-2.0/schemas/" || true

echo -e "${YELLOW}Creating custom AppRun script...${NC}"
cat > "${APPDIR}/AppRun" << 'EOF'
#!/bin/bash

HERE="$(dirname "$(readlink -f "${0}")")"

export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
export GTK_DATA_PREFIX="${HERE}/usr"
export GSETTINGS_SCHEMA_DIR="${HERE}/usr/share/glib-2.0/schemas:${GSETTINGS_SCHEMA_DIR}"

unset GDK_PIXBUF_MODULE_FILE
unset GDK_PIXBUF_MODULEDIR

exec "${HERE}/usr/bin/lsfg-vk-ui" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

echo -e "${YELLOW}Downloading appimagetool...${NC}"
wget -qc "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"
chmod +x appimagetool-x86_64.AppImage

echo -e "${YELLOW}Generating AppImage using appimagetool...${NC}"
ARCH=x86_64 ./appimagetool-x86_64.AppImage "${APPDIR}" "${FINAL_APPIMAGE_NAME}"

echo -e "${YELLOW}Cleaning up build directories...${NC}"
rm -rf AppDir appimagetool-x86_64.AppImage

echo ""
echo -e "${GREEN}✓ AppImage built successfully: ${FINAL_APPIMAGE_NAME}${NC}"
echo -e "${GREEN}✓ File size: $(du -h "${FINAL_APPIMAGE_NAME}" | cut -f1)${NC}"
echo ""
echo -e "${BLUE}Usage:${NC}"
echo "  chmod +x ${FINAL_APPIMAGE_NAME}"
echo "  ./${FINAL_APPIMAGE_NAME}"
