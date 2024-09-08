// SPDX-License-Identifier: GPL-2.0

#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/jiffies.h>

static int __init jiffies_test_init(void)
{
    unsigned long j = 1000;
    unsigned int msecs;
    unsigned int nsecs;

    pr_info("jiffies_test: Module init\n");


    // Convert jiffies to milliseconds using kernel function
    msecs = jiffies_to_msecs(j);

    pr_info("jiffies_test: Jiffies: %lu, Milliseconds: %u\n", j, msecs);

    // Convert jiffies to milliseconds using kernel function
    nsecs = jiffies_to_usecs(j+1);

    pr_info("jiffies_test: Jiffies: %lu, Microseconds: %u\n", j+1, nsecs);


    // Here, you could call the equivalent Rust function and compare the results
    // Example: rust_result = rust_jiffies_to_msecs(j);
    // pr_info("Rust conversion: %u, C conversion: %u\n", rust_result, msecs);

    return 0;
}

static void __exit jiffies_test_exit(void)
{
    pr_info("jiffies_test: Module exit\n");
}

module_init(jiffies_test_init);
module_exit(jiffies_test_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Your Name");
MODULE_DESCRIPTION("Test module for jiffies_to_msecs conversion");
