// kprobe_setup

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kprobes.h>

int initialize_kprobe(void);
void cleanup_kprobe(void);

static struct kprobe kp = {
    .symbol_name = "vfs_open",  // Example: Intercept the 'vfs_open' syscall
};

// Adjusted function signature to match kprobe_pre_handler_t
static int handler_pre(struct kprobe *p, struct pt_regs *regs) {
    struct path* path;
    const char *pathname;
    struct inode* inode;

    path = (struct path *) regs->di;
    pathname = path->dentry->d_name.name;
    inode = path->dentry->d_inode;
    printk(KERN_INFO "Kprobe: vfs_open called on: %s  with inode: %d \n",pathname, inode->i_ino);
    
    return 0;
}

// Declared static to avoid warnings about missing prototypes
int initialize_kprobe(void) {
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
EXPORT_SYMBOL(initialize_kprobe);

// Declared static to avoid warnings about missing prototypes
void cleanup_kprobe(void) {
    unregister_kprobe(&kp);
    printk(KERN_INFO "Kprobe unregistered\n");
}
EXPORT_SYMBOL(cleanup_kprobe);

//module_init(initialize_kprobe);
//module_exit(cleanup_kprobe);

MODULE_LICENSE("GPL");
