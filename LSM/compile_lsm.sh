#!/bin/sh
# Early boot script to compile and load the LSM module

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
        echo "LSM_Installer: Starting compile_and_load_lsm script" > /dev/kmsg

        # Mount the root filesystem as read-write
        mount -o remount,rw /

        # Define module source directory
        MODULE_SRC_DIR="/usr/src/lsm_module"

        # Ensure module source directory exists
        if [ ! -d "$MODULE_SRC_DIR" ]; then
            echo "LSM_Installer: Module source directory not found. Exiting." > /dev/kmsg
            exit 1
        fi

        # Navigate to the directory with the module source
        cd "$MODULE_SRC_DIR"

        # Compile the module using the headers from the root filesystem
        make

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Install and load the module
            insmod my_lsm.ko
            echo "LSM_Installer: Custom LSM module installed and loaded" > /dev/kmsg
        else
            echo "LSM_Installer: Module compilation failed. Exiting." > /dev/kmsg
            exit 1
        fi
        ;;
esac
