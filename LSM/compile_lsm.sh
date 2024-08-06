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

        # Mount the root filesystem as read-write
        mount -o remount,rw /

        # Ensure module source directory exists
        MODULE_SRC_DIR="/usr/src/lsm_module"

        if [ ! -d "$MODULE_SRC_DIR" ]; then
            log_message "LSM_Installer: Module source directory not found. Exiting."
            exit 1
        fi

        # Navigate to the directory with the module source
        cd "$MODULE_SRC_DIR"

        # Compile the module using the headers from the root filesystem
        make > /tmp/lsm_compile.log 2>&1

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Install and load the module
            insmod my_lsm.ko > /tmp/lsm_compile.log 2>&1
            log_message "LSM_Installer: Custom LSM module installed and loaded"
        else
            log_message "LSM_Installer: Module compilation failed. Check /tmp/lsm_compile.log for details."
            exit 1
        fi
        ;;
esac
