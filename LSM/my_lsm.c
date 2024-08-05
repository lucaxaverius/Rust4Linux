#include <linux/module.h>
#include <linux/init.h>
#include <linux/security.h>
#include <linux/fs.h>
#include <linux/sched.h>
#include <linux/cred.h>
#include <linux/lsm_hooks.h>

static int my_file_open(struct inode *inode, struct file *file)
{
    const struct cred *cred;
    uid_t uid;
    const char *process_name;

    // Get the current process credentials
    cred = current_cred();
    uid = cred->uid.val;
    process_name = current->comm;

    pr_info("My LSM: File open intercepted by process %s (UID: %d)\n", process_name, uid);

    // Implement your access control logic here
    // Example: Deny access if the user ID is 1000 (non-root user)
    if (uid == 1000) {
        pr_info("My LSM: Access denied for process %s (UID: %d)\n", process_name, uid);
        //return -EACCES;
    }

    return 0;
}

static int my_inode_permission(struct inode *inode, int mask)
{
    pr_info("My LSM: Inode permission intercepted\n");
    // Implement your access control logic here
    return 0;
}

static struct security_hook_list my_hooks[]  __lsm_ro_after_init = {
    LSM_HOOK_INIT(file_open, my_file_open), 
    LSM_HOOK_INIT(inode_permission, my_inode_permission),
};

static int __init my_lsm_init(void)
{
    pr_info("My LSM: Initializing...\n"); 
    security_add_hooks(my_hooks, ARRAY_SIZE(my_hooks), "my_lsm");
    return 0;
}

security_initcall(my_lsm_init);

static void __exit my_lsm_exit(void)
{
    pr_info("My LSM: Exiting...\n");
    // Cleanup code if necessary
}

module_exit(my_lsm_exit);

MODULE_LICENSE("GPL");
MODULE_DESCRIPTION("Basic LSM Example with Multiple Hooks");
MODULE_AUTHOR("Luca Saverio Esposito");
