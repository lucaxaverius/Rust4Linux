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
        # Mount the root filesystem as read-write
        mount -o remount,rw /

        # Ensure the build environment is set up
        # Ensure the kernel headers and make tools are available in initramfs
        if [ ! -d /lib/modules/$(uname -r)/build ]; then
            echo "Kernel source not found. Exiting."
            exit 1
        fi

        # Create a working directory
        mkdir -p /root/lsm_module
        cd /root/lsm_module

        # Copy the module source files from their location to the initramfs
        cp /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM/Makefile .
        cp /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM/my_lsm.c .

        # Compile the module
        make LLVM=1 -C /lib/modules/$(uname -r)/build M=$PWD modules

        # Check if the module compiled successfully
        if [ -f my_lsm.ko ]; then
            # Move the compiled module to the appropriate location
            mkdir -p /lib/modules/$(uname -r)/kernel/security
            cp my_lsm.ko /lib/modules/$(uname -r)/kernel/security/
            insmod my_lsm.ko
            echo "Custom LSM correctly installed"

        else
            echo "Module compilation failed. Exiting."
            exit 1
        fi
        ;;
esac
