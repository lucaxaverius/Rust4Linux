# SPDX-License-Identifier: GPL-2.0

KDIR ?= /lib/modules/$(shell uname -r)/build
obj-m := sec_module.o 


# Build everything (both C and Rust code)
all:
	$(MAKE) LLVM=1 -C $(KDIR) M=$(PWD) modules
	gcc -o sec_tool sec_tools.c

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
	rm sec_tool