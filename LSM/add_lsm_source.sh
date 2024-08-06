#!/bin/sh
# Hook script to copy LSM source files into initramfs

PREREQ=""

prereqs() {
    echo "$PREREQ"
}

case "$1" in
    prereqs)
        prereqs
        exit 0
        ;;
    *)
        echo "LSM_Installer: Initialization" > /dev/kmsg

        # Define source and destination directories
        SRC_DIR="/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"
        DEST_DIR="${DESTDIR}/usr/src/lsm_module"

        # Copy source files to initramfs
        mkdir -p "$DEST_DIR"
        cp -r "$SRC_DIR"/* "$DEST_DIR"

        echo "LSM_Installer: LSM source files copied to initramfs" > /dev/kmsg
        ;;
esac
