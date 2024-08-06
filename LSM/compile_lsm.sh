#!/bin/sh
# Early boot script to compile and load the LSM module

PREREQ=""

prereqs() {
    echo "$PREREQ"
}

log_message() {
    echo "$1" >> /mnt/tmp/lsm_installer.log
    echo "$1" > /dev/kmsg
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
        mount -o remount,rw /

        # Mount the root filesystem if needed
        mount /dev/vda3 /mnt

        # Ensure module source directory exists
        MODULE_SRC_DIR="/mnt/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"

        if [ ! -d "$MODULE_SRC_DIR" ]; then
            log_message "LSM_Installer: Module source directory not found. Exiting."
            exit 1
        fi

        # Ensure kernel headers are available
        KERNEL_HEADERS_DIR="/lib/modules/$(shell uname -r)/build"

        if [ ! -d "$KERNEL_HEADERS_DIR" ]; then
            log_message "LSM_Installer: Kernel headers not found. Exiting."
            exit 1
        fi

        # Navigate to the directory with the module source
        cd "$MODULE_SRC_DIR"

        # Compile the module using the headers from the root filesystem
        make > /mnt/tmp/lsm_compile.log 2>&1

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Install and load the module
            insmod my_lsm.ko > /mnt/tmp/lsm_compile.log 2>&1
            log_message "LSM_Installer: Custom LSM module installed and loaded"
        else
            log_message "LSM_Installer: Module compilation failed. Check /tmp/lsm_compile.log for details."
            exit 1
        fi

        # Unmount filesystems
        umount /mnt
        umount /sys
        umount /proc
        ;;
esac
