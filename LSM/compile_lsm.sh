#!/bin/sh
# Early boot script to compile and load the LSM module

PREREQ=""

prereqs() {
    echo "$PREREQ"
}

log_message() {
    echo "$1" >> /tmp/lsm_installer.log
    echo "$1" > /dev/kmsg
}

cleanup() {
    # Unmount filesystems
    umount /sys 2>/dev/null
    umount /proc 2>/dev/null
}

case "$1" in
    prereqs)
        prereqs
        exit 0
        ;;
    *)
        log_message "LSM_Installer: Starting compile_and_load_lsm script"

        # Mount necessary filesystems
        mount -t proc proc /proc
        mount -t sysfs sysfs /sys

        # Mount the root filesystem on /
        if ! mount /dev/vda3 /; then
            log_message "LSM_Installer: Failed to mount /dev/vda3 on /. Exiting."
            cleanup
            exit 1
        fi

        # Ensure module source directory exists
        MODULE_SRC_DIR="/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"

        if [ ! -d "$MODULE_SRC_DIR" ]; then
            log_message "LSM_Installer: Module source directory not found. Exiting."
            cleanup
            exit 1
        fi

        # Ensure kernel headers are available
        KERNEL_HEADERS_DIR="/lib/modules/$(uname -r)/build"

        if [ ! -d "$KERNEL_HEADERS_DIR" ]; then
            log_message "LSM_Installer: Kernel headers not found. Exiting."
            cleanup
            exit 1
        fi

        # Navigate to the directory with the module source
        cd "$MODULE_SRC_DIR" || { log_message "LSM_Installer: Failed to change directory to $MODULE_SRC_DIR. Exiting."; cleanup; exit 1; }

        # Compile the module using the headers from the root filesystem
        if ! make > /tmp/lsm_compile.log 2>&1; then
            log_message "LSM_Installer: Module compilation failed. Check /tmp/lsm_compile.log for details."
            cleanup
            exit 1
        fi

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Install and load the module
            if ! insmod my_lsm.ko > /tmp/lsm_compile.log 2>&1; then
                log_message "LSM_Installer: Failed to load the module. Check /tmp/lsm_compile.log for details."
                cleanup
                exit 1
            fi
            log_message "LSM_Installer: Custom LSM module installed and loaded"
        else
            log_message "LSM_Installer: Module compilation did not produce my_lsm.ko. Check /tmp/lsm_compile.log for details."
            cleanup
            exit 1
        fi

        # Unmount filesystems
        cleanup
        ;;
esac
