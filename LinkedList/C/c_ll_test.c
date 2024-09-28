// SPDX-License-Identifier: GPL-2.0

#include <linux/init.h>
#include <linux/module.h>
#include <linux/kernel.h>
#include <linux/list.h>
#include <linux/slab.h>
#include <linux/ktime.h>

MODULE_LICENSE("GPL v2");
MODULE_AUTHOR("Luca Saverio Esposito");
MODULE_DESCRIPTION("C version of linked list operations");
MODULE_VERSION("0.1");

#define LIST_SIZE 10000000

struct my_list_item {
    struct list_head list;
    u32 data;
};

static struct list_head head;

static int __init linked_list_test_init(void) {
    pr_info("c_ll_test: Starting Linked List Operations Test in C...\n");

    // Initialize list head
    INIT_LIST_HEAD(&head);

    // Measure adding elements
    ktime_t start_add = ktime_get();
    for (int i = 0; i < LIST_SIZE; i++) {
        struct my_list_item *item = kmalloc(sizeof(struct my_list_item), GFP_KERNEL);
        if (!item) {
            pr_err("c_ll_test: Failed to allocate memory for list item\n");
            return -ENOMEM;
        }
        item->data = i;
        INIT_LIST_HEAD(&item->list);
        list_add_tail(&item->list, &head);
    }
    ktime_t end_add = ktime_get();
    s64 duration_add = ktime_ms_delta(end_add, start_add);
    pr_info("c_ll_test: Time taken to add %d elements: %lld ms\n", LIST_SIZE, duration_add);

    // Check if the list is empty after adding LIST_SIZE items
    if (list_empty(&head)) {
        pr_err("c_ll_test: List is empty after adding elements!\n");
        return -EINVAL;
    }
    pr_info("c_ll_test: List is not empty after adding %d items.\n", LIST_SIZE);

    // Measure iterating over elements
    ktime_t start_iter = ktime_get();
    struct my_list_item *item;
    list_for_each_entry(item, &head, list) {
        item->data += 1;
        //pr_info("Iterating item with data: %u\n", item->data);
    }
    ktime_t end_iter = ktime_get();
    s64 duration_iter = ktime_ms_delta(end_iter, start_iter);
    pr_info("c_ll_test: Time taken to iterate over %d elements: %lld ms\n", LIST_SIZE, duration_iter);

    // Measure replacing elements
    ktime_t start_replace = ktime_get();
    struct my_list_item *tmp;
    int i = 1;
    list_for_each_entry_safe(item, tmp, &head, list) {
        struct my_list_item *replacement = kmalloc(sizeof(struct my_list_item), GFP_KERNEL);
        if (!replacement) {
            pr_err("c_ll_test: Failed to allocate memory for replacement item\n");
            return -ENOMEM;
        }
        replacement->data = i++;
        INIT_LIST_HEAD(&replacement->list);
        list_replace(&item->list, &replacement->list);
        kfree(item);
    }
    ktime_t end_replace = ktime_get();
    s64 duration_replace = ktime_ms_delta(end_replace, start_replace);
    pr_info("c_ll_test: Time taken to replace %d elements: %lld ms\n", LIST_SIZE, duration_replace);

    // Measure removing elements
    ktime_t start_del = ktime_get();
    list_for_each_entry_safe(item, tmp, &head, list) {
        list_del(&item->list);
        kfree(item);
    }
    ktime_t end_del = ktime_get();
    s64 duration_del = ktime_ms_delta(end_del, start_del);
    pr_info("c_ll_test: Time taken to remove %d elements: %lld ms\n", LIST_SIZE, duration_del);

    // Check if the list is empty after removing all the items
    if (list_empty(&head)) {
        pr_info("c_ll_test: List is empty after removing all the items.\n");
    } else {
        pr_err("c_ll_test: List is not empty after removing all the items!\n");
    }

    pr_info("c_ll_test: Linked List Operations Test Completed in C.\n");
    return 0;
}

static void __exit linked_list_test_exit(void) {
    pr_info("c_ll_test: Module unloaded\n");
}

module_init(linked_list_test_init);
module_exit(linked_list_test_exit);
