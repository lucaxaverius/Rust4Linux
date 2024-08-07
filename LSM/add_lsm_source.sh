#!/bin/sh

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
        echo "LSM_Installer: Starting the initialization phase" 

        # Copy necessary tools
        copy_exec /usr/bin/make /usr/bin
        copy_exec /usr/bin/gcc /usr/bin

        # Copy kernel headers
        mkdir -p "${DESTDIR}/lib/modules/$(uname -r)/build"
        cp -r /lib/modules/$(uname -r)/build/* "${DESTDIR}/lib/modules/$(uname -r)/build/"

        # Copy the module source code
        mkdir -p "${DESTDIR}/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"
        cp -r /home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM/* "${DESTDIR}/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM/"
        ;;
esac
