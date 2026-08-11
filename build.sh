#!/bin/sh
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "WARNING: NDK not set."
else
    NDK_PATH="${ANDROID_NDK_HOME:-$ANDROID_NDK_ROOT}"
    LINKER_PATH="$NDK_PATH/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android34-clang"
    if [ -f "$LINKER_PATH" ]; then
        export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$LINKER_PATH"
    fi
fi

cd storagerefresh-rust/daemon
cargo build --release --target aarch64-linux-android
cd "$SCRIPT_DIR"

verify_android_arm64_elf() {
    path="$1"
    if [ ! -f "$path" ]; then
        echo "ERROR: Missing binary: $path"
        false
    fi
    magic="$(od -An -tx1 -N4 "$path" | tr -d ' \n')"
    machine="$(od -An -tx2 -j18 -N2 "$path" | tr -d ' \n')"
    if [ "$magic" != "7f454c46" ]; then
        echo "ERROR: Invalid ELF magic: $path"
        false
    fi
    if [ "$machine" != "00b7" ] && [ "$machine" != "b700" ]; then
        echo "ERROR: Binary is not AArch64 ELF: $path (e_machine=$machine)"
        false
    fi
}

SRC_PATH="storagerefresh-rust/target/aarch64-linux-android/release/storagerefresh-rust"
if [ -f "$SRC_PATH" ]; then
    STAGING_DIR="$SCRIPT_DIR/staging_zip"
    rm -rf "$STAGING_DIR"
    mkdir -p "$STAGING_DIR"

    # Copy from our magisk module structure
    cp -R storagerefresh-magisk-module/* "$STAGING_DIR/"

    # Overwrite binary just to be safe
    cp "$SRC_PATH" "$STAGING_DIR/system/bin/storagerefresh-rust"

    # Strip using NDK strip if possible
    STRIP_PATH="${ANDROID_NDK_HOME:-$ANDROID_NDK_ROOT}/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip"
    if [ -x "$STRIP_PATH" ]; then
        "$STRIP_PATH" --strip-all "$STAGING_DIR/system/bin/storagerefresh-rust"
    fi

    verify_android_arm64_elf "$STAGING_DIR/system/bin/storagerefresh-rust"

    find "$STAGING_DIR" -type f \( -name '*.sh' -o -name '*.prop' -o -name '*.conf' -o -name '*.md' -o -name '*.rule' -o -name '*.html' -o -name '*.css' -o -name '*.js' -o -name update-binary \) -exec sed -i 's/\r$//' {} +

    echo "Zipping module..."
    rm -f StorageRefresh-Rust.zip
    if command -v 7z >/dev/null 2>&1; then
        (cd "$STAGING_DIR" && 7z a -tzip "$SCRIPT_DIR/StorageRefresh-Rust.zip" ./*)
    else
        (cd "$STAGING_DIR" && zip -r "$SCRIPT_DIR/StorageRefresh-Rust.zip" .)
    fi
    rm -rf "$STAGING_DIR"
    echo "Build complete."
else
    echo "ERROR: Target binary missing."
    false
fi
