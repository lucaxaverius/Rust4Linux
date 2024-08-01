# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/$(shell uname -r)/build

# Build everything (both C and Rust code)
all:
	$(MAKE) LLVM=1 -C $(KDIR) M=$(PWD)

modules_install: all
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
