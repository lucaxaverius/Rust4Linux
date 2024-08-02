# Directory where the kernel build system is located
KDIR := /lib/modules/$(shell uname -r)/build

# Source and Object files
obj-m := kprobe_setup.o

# Default target
all:
	$(MAKE) LLVM=1 -C $(KDIR) M=$(PWD) modules

# Clean up generated files
clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean
