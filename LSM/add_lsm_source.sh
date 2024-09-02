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
        # Ensure destination directories exist
        mkdir -p "${DESTDIR}/usr/bin"
        mkdir -p "${DESTDIR}/lib/modules/$(uname -r)/build"
        mkdir -p "${DESTDIR}/lib/modules/$(uname -r)/build/include"
        mkdir -p "${DESTDIR}/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"

        # Copy necessary tools
        cp /usr/bin/make "${DESTDIR}/usr/bin/"
        cp /usr/bin/gcc "${DESTDIR}/usr/bin/"

        # Copy essential kernel headers
        KERNEL_BUILD_DIR="/lib/modules/$(uname -r)/build"
        if [ -d "${KERNEL_BUILD_DIR}" ]; then
            cp "${KERNEL_BUILD_DIR}/Makefile" "${DESTDIR}/lib/modules/$(uname -r)/build/"
            cp -r "${KERNEL_BUILD_DIR}/include" "${DESTDIR}/lib/modules/$(uname -r)/build/"
            cp -r "${KERNEL_BUILD_DIR}/scripts" "${DESTDIR}/lib/modules/$(uname -r)/build/"
        else
            echo "Kernel build directory not found: ${KERNEL_BUILD_DIR}"
            exit 1
        fi

        # Copy the module source code
        MODULE_SRC_DIR="/home/rustxave/Scrivania/Rust-Modules/Rust4Linux/LSM"
        if [ -d "${MODULE_SRC_DIR}" ]; then
            cp -r "${MODULE_SRC_DIR}"/* "${DESTDIR}${MODULE_SRC_DIR}/"
        else
            echo "Module source directory not found: ${MODULE_SRC_DIR}"
            exit 1
        fi
        ;;
esac
