#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kprobes.h>

static struct kprobe kp = {
    .symbol_name = "vfs_open",  // Example: Intercept the 'vfs_open' syscall
};

// Adjusted function signature to match kprobe_pre_handler_t
static int handler_pre(struct kprobe *p, struct pt_regs *regs) {
    printk(KERN_INFO "Kprobe: vfs_open called\n");
    return 0;
}

// Declared static to avoid warnings about missing prototypes
static int initialize_kprobe(void) {
    kp.pre_handler = handler_pre;
    int ret = register_kprobe(&kp);
    if (ret < 0) {
        printk(KERN_INFO "Failed to register kprobe\n");
        return ret;
    } else {
        printk(KERN_INFO "Kprobe registered\n");
    }
    return 0;
}

// Declared static to avoid warnings about missing prototypes
static void cleanup_kprobe(void) {
    unregister_kprobe(&kp);
    printk(KERN_INFO "Kprobe unregistered\n");
}

module_init(initialize_kprobe);
module_exit(cleanup_kprobe);

MODULE_LICENSE("GPL");
