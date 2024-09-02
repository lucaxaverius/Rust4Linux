#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/fs.h>
#include <linux/uaccess.h>
#include <linux/cdev.h>

#define DEVICE_NAME "secrules"
#define MAX_RULES 100
#define MAX_RULE_LENGTH 100

static int major_number;
static char rules[MAX_RULES][MAX_RULE_LENGTH];
static int rule_count = 0;
static struct cdev c_dev;
static struct class *cl;

static ssize_t device_read(struct file *filp, char *buffer, size_t len, loff_t *offset)
{
    int i;
    int remaining_size = len;
    char *msg;
    char temp_buffer[MAX_RULES * MAX_RULE_LENGTH];
    int buffer_offset = 0;

    if (rule_count == 0) {
        return 0;
    }

    for (i = 0; i < rule_count; i++) {
        msg = rules[i];
        snprintf(temp_buffer + buffer_offset, MAX_RULE_LENGTH, "%s\n", msg);
        buffer_offset += strlen(msg) + 1;
    }

    if (*offset >= buffer_offset) {
        return 0;
    }

    remaining_size = min((size_t)(buffer_offset - *offset), len);

    if (copy_to_user(buffer, temp_buffer + *offset, remaining_size)) {
        return -EFAULT;
    }

    *offset += remaining_size;
    return remaining_size;
}

static ssize_t device_write(struct file *filp, const char *buffer, size_t len, loff_t *off)
{
    if (rule_count >= MAX_RULES || len > MAX_RULE_LENGTH) {
        return -EINVAL;
    }

    if (copy_from_user(rules[rule_count], buffer, len)) {
        return -EFAULT;
    }

    rules[rule_count][len] = '\0'; // Null-terminate the string
    rule_count++;
    return len;
}

static int device_open(struct inode *inode, struct file *file)
{
    return 0;
}

static int device_release(struct inode *inode, struct file *file)
{
    return 0;
}

static struct file_operations fops =
{
    .open = device_open,
    .release = device_release,
    .read = device_read,
    .write = device_write,
};

static int __init sec_module_init(void)
{
    major_number = register_chrdev(0, DEVICE_NAME, &fops);
    if (major_number < 0) {
        printk(KERN_ALERT "Failed to register character device\n");
        return major_number;
    }

    cl = class_create("secclass");
    device_create(cl, NULL, MKDEV(major_number, 0), NULL, DEVICE_NAME);

    printk(KERN_INFO "Security module loaded with device major number %d\n", major_number);
    return 0;
}

static void __exit sec_module_exit(void)
{
    device_destroy(cl, MKDEV(major_number, 0));
    class_destroy(cl);
    unregister_chrdev(major_number, DEVICE_NAME);
    printk(KERN_INFO "Security module unloaded\n");
}

module_init(sec_module_init);
module_exit(sec_module_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Luca Saverio Esposito");
MODULE_DESCRIPTION("A simple security module");
