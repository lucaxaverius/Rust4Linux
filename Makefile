# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/$(shell uname -r)/build

# Build the C module
c:
	$(MAKE) -C $(KDIR) M=$(PWD) c

# Build the Rust module
rust:
	$(MAKE) -C $(KDIR) M=$(PWD) rust

# Build everything
all: c rust

modules_install: all
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
