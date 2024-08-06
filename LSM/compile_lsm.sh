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


case "$1" in
    prereqs)
        prereqs
        exit 0
        ;;
    *)
        log_message "LSM_Installer: Starting compile_and_load_lsm script"

        # Ensure root filesystem is mounted as read-write
        if ! mount -o remount,rw /; then
            log_message "LSM_Installer: Failed to remount root filesystem as read-write. Exiting."
            exit 1
        fi

        # Ensure module source directory exists
        MODULE_SRC_DIR="/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"

        if [ ! -d "$MODULE_SRC_DIR" ]; then
            log_message "LSM_Installer: Module source directory not found. Exiting."
            exit 1
        fi

        # Ensure kernel headers are available
        KERNEL_HEADERS_DIR="/lib/modules/$(uname -r)/build"

        if [ ! -d "$KERNEL_HEADERS_DIR" ]; then
            log_message "LSM_Installer: Kernel headers not found. Exiting."
            exit 1
        fi

        # Navigate to the directory with the module source
        cd "$MODULE_SRC_DIR" || { log_message "LSM_Installer: Failed to change directory to $MODULE_SRC_DIR. Exiting."; exit 1; }

        # Compile the module using the headers from the root filesystem
        if ! make > /tmp/lsm_compile.log 2>&1; then
            log_message "LSM_Installer: Module compilation failed. Check /tmp/lsm_compile.log for details."
            exit 1
        fi

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Install and load the module
            if ! insmod my_lsm.ko > /tmp/lsm_compile.log 2>&1; then
                log_message "LSM_Installer: Failed to load the module. Check /tmp/lsm_compile.log for details."
                exit 1
            fi
            log_message "LSM_Installer: Custom LSM module installed and loaded"
        else
            log_message "LSM_Installer: Module compilation did not produce my_lsm.ko. Check /tmp/lsm_compile.log for details."
            exit 1
        fi

        # Unmount filesystems (if applicable)
        ;;
esac
