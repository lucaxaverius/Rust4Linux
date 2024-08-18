#include <linux/module.h>
#include <linux/fs.h>
#include <linux/device.h>
#include <linux/cdev.h>

#define DEVICE_NAME "sec_device"
#define CLASS_NAME "sec_class"

static int major_number;
static struct class* sec_class = NULL;
static struct device* sec_device = NULL;
static struct cdev sec_cdev;

extern ssize_t rust_read(struct file *file, char *buffer, size_t len, loff_t *offset);
extern ssize_t rust_write(struct file *file, const char *buffer, size_t len, loff_t *offset);

static struct file_operations fops = {
    .owner = THIS_MODULE,
    .read = rust_read,
    .write = rust_write,
};

int create_device(void) {
    major_number = register_chrdev(0, DEVICE_NAME, &fops);
    if (major_number < 0) {
        printk(KERN_ALERT "Failed to register a major number\n");
        return major_number;
    }

    sec_class = class_create(THIS_MODULE, CLASS_NAME);
    if (IS_ERR(sec_class)) {
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "Failed to register device class\n");
        return PTR_ERR(sec_class);
    }

    sec_device = device_create(sec_class, NULL, MKDEV(major_number, 0), NULL, DEVICE_NAME);
    if (IS_ERR(sec_device)) {
        class_destroy(sec_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "Failed to create the device\n");
        return PTR_ERR(sec_device);
    }

    cdev_init(&sec_cdev, &fops);
    if (cdev_add(&sec_cdev, MKDEV(major_number, 0), 1) < 0) {
        device_destroy(sec_class, MKDEV(major_number, 0));
        class_destroy(sec_class);
        unregister_chrdev(major_number, DEVICE_NAME);
        printk(KERN_ALERT "Failed to add cdev\n");
        return -1;
    }

    printk(KERN_INFO "Security device registered: /dev/%s with major number %d\n", DEVICE_NAME, major_number);
    return 0;
}

void remove_device(void) {
    cdev_del(&sec_cdev);
    device_destroy(sec_class, MKDEV(major_number, 0));
    class_unregister(sec_class);
    class_destroy(sec_class);
    unregister_chrdev(major_number, DEVICE_NAME);
    printk(KERN_INFO "Security device unregistered\n");
}

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Your Name");
MODULE_DESCRIPTION("A minimal C device registration for Rust integration");
MODULE_VERSION("0.1");
