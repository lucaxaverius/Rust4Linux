// kprobe_setup.c

#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/kprobes.h>
#include <linux/cred.h> // For current_cred()

int initialize_kprobe(void);
void cleanup_kprobe(void);

extern bool check_user_id(u32 user_id);  // Rust function to check if UID is blacklisted

static struct kprobe kp = {
    .symbol_name = "vfs_open",  // Example: Intercept the 'vfs_open' syscall
};

// Adjusted function signature to match kprobe_pre_handler_t
static int handler_pre(struct kprobe *p, struct pt_regs *regs) {
    bool is_blacklisted;
    struct path* path;
    const char *pathname;
    struct inode* inode;
    u32 user_id;

    path = (struct path *) regs->di;
    pathname = path->dentry->d_name.name;
    inode = path->dentry->d_inode;

    
    // Get the current task's credentials
    const struct cred *cred = current_cred();
    user_id = from_kuid(&init_user_ns, cred->uid); // Convert UID to a numeric value


    // Call the Rust function to handle UID checking
    is_blacklisted = check_user_id(user_id);

    if (is_blacklisted) {
        //printk(KERN_WARNING "Kprobe: Blacklisted user %d detected!\n",user_id);
        // Add additional handling for blacklisted user ID if necessary
        printk(KERN_INFO "rust_kprobes: vfs_open called on: %s with inode: %d\n", pathname, inode->i_ino);

    }

    return 0;
}

int initialize_kprobe(void) {
    kp.pre_handler = handler_pre;
    int ret = register_kprobe(&kp);
    if (ret < 0) {
        printk(KERN_ERR "rust_kprobes: Failed to register kprobe\n");
        return ret;
    } else {
        printk(KERN_INFO "rust_kprobes: Kprobe registered\n");
    }
    return 0;
}
EXPORT_SYMBOL(initialize_kprobe);

void cleanup_kprobe(void) {
    unregister_kprobe(&kp);
    printk(KERN_INFO "rust_kprobes: Kprobe unregistered\n");
}
EXPORT_SYMBOL(cleanup_kprobe);

MODULE_LICENSE("GPL");
