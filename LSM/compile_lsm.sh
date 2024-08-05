#!/bin/sh
# Early boot script to compile the LSM module

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
        echo "LSM_Installer: Starting compile_lsm script" > /dev/kmsg

        # Mount the root filesystem as read-write
        mount -o remount,rw /

       # Ensure the build environment is set up
        if [ ! -d /lib/modules/$(uname -r)/build ]; then
            # Copy kernel headers to initramfs
            if [ -d "/lib/modules/$(uname -r)/build" ]; then
                mkdir -p ${DESTDIR}/lib/modules/$(uname -r)
                cp -r /lib/modules/$(uname -r)/build ${DESTDIR}/lib/modules/$(uname -r)
                echo "LSM_Installer: Kernel source correctly copied." > /dev/kmsg
            else
                echo "LSM_Installer: Kernel source not found. Exiting." > /dev/kmsg
                exit 1
            fi
        fi

        # Navigate to the directory with the module source
        cd /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM

        # Compile the module
        make -C /lib/modules/$(uname -r)/build M=$PWD modules

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Move the compiled module to the appropriate location
            insmod my_lsm.ko
            echo "LSM_Installer: Custom LSM module installed and loaded" > /dev/kmsg
        else
            echo "LSM_Installer: Module compilation failed. Exiting." > /dev/kmsg
            exit 1
        fi
        ;;
esac
