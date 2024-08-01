# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/$(shell uname -r)/build

# Targets for building C and Rust code
all: build_c build_rust

build_c:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

build_rust:
	cargo build --release

modules_install: all
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
	cargo clean
