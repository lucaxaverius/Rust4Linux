// kprobe_setup.c
#include <linux/module.h>
#include <linux/kprobes.h>

static struct kprobe kp = {
    .symbol_name = "vfs_open",  // Example: Intercept the 'vfs_open' syscall
};

static int handler_pre(struct pt_regs *regs) {
    printk(KERN_INFO "Kprobe: vfs_open called\n");
    return 0;
}

static int __init kprobe_init(void) {
    kp.pre_handler = handler_pre;
    int ret = register_kprobe(&kp);
    if (ret < 0) {
        printk(KERN_INFO "Failed to register kprobe\n");
        return ret;
    }
    printk(KERN_INFO "Kprobe registered\n");
    return 0;
}

static void __exit kprobe_exit(void) {
    unregister_kprobe(&kp);
    printk(KERN_INFO "Kprobe unregistered\n");
}

module_init(kprobe_init);
module_exit(kprobe_exit);

MODULE_LICENSE("GPL");
