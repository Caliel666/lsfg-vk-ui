# --- 4. Manually Copy Libraries (including libadwaita) ---
echo -e "${YELLOW}Manually copying required libraries...${NC}"

# Function to copy a library and its direct dependencies
copy_library_with_deps() {
    local lib_path="$1"
    local dest_dir="$2"
    
    # Check if the library is a symbolic link and resolve it
    if [ -L "$lib_path" ]; then
        lib_path=$(readlink -f "$lib_path")
    fi

    # Copy the library itself
    if [ -f "$lib_path" ]; then
        # Check if the file already exists in the destination to prevent redundant copies
        if [ ! -f "${dest_dir}/$(basename "$lib_path")" ]; then
            cp "$lib_path" "$dest_dir"
            echo "  Copied: $(basename "$lib_path")"
        fi
        
        # Get its direct dependencies
        # Filter out common system libraries that are expected to be present
        # and explicitly exclude gdk-pixbuf related libraries.
        # Also exclude libgdk-4.so if it's explicitly being problematic from a system path,
        # but it should be pulled by libgtk-4.so's dependencies.
        ldd "$lib_path" 2>/dev/null | grep '=>' | grep -v 'libgdk_pixbuf' | grep -v 'libgdk-pixbuf' | grep -v 'libgdk-4.so' | grep -v 'libgtk-4.so' | awk '{print $3}' | while read -r dep_lib; do
            if [[ "$dep_lib" == /* ]] && [ -f "$dep_lib" ]; then
                # Only copy if it's not already in the AppDir/usr/lib
                if [ ! -f "${dest_dir}/$(basename "$dep_lib")" ]; then
                    copy_library_with_deps "$dep_lib" "$dest_dir"
                fi
            fi
        done
    else
        echo -e "${YELLOW}Warning: Library not found (or already handled) - $lib_path${NC}"
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
REQUIRED_LIBS=(
    "/usr/lib/x86_64-linux-gnu/libadwaita-1.so"
    "/usr/lib/x86_64-linux-gnu/libgtk-4.so"
    "/usr/lib/x86_64-linux-gnu/libgio-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libglib-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libcairo.so"
    "/usr/lib/x86_64-linux-gnu/libpango-1.0.so"
    "/usr/lib/x86_64-linux-gnu/libharfbuzz.so"
    "/usr/lib/x86_64-linux-gnu/libfontconfig.so"
    "/usr/lib/x86_64-linux-gnu/libfreetype.so"
    "/usr/lib/x86_64-linux-gnu/libgobject-2.0.so"
    "/usr/lib/x86_64-linux-gnu/libepoxy.so"
    "/usr/lib/x86_64-linux-gnu/libgmodule-2.0.so"
)

for lib in "${REQUIRED_LIBS[@]}"; do
    # Call the function; the function itself handles the 'not found' warning.
    # The '|| true' ensures the loop continues even if 'copy_library_with_deps'
    # prints a warning, preventing 'set -e' from exiting.
    copy_library_with_deps "$lib" "${APPDIR}/usr/lib" || true 
done

# --- 4.1 Copy GSettings Schemas ---
echo -e "${YELLOW}Copying GSettings schemas...${NC}"
cp /usr/share/glib-2.0/schemas/gschemas.compiled "${APPDIR}/usr/share/glib-2.0/schemas/" || true