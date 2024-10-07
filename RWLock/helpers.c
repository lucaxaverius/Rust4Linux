// SPDX-License-Identifier: GPL-2.0
/*
 * Non-trivial C macros cannot be used in Rust. Similarly, inlined C functions
 * cannot be called either. This file explicitly creates functions ("helpers")
 * that wrap those so that they can be called from Rust.
 *
 * Even though Rust kernel modules should never use the bindings directly, some
 * of these helpers need to be exported because Rust generics and inlined
 * functions may not get their code generated in the crate where they are
 * defined. Other helpers, called from non-inline functions, may not be
 * exported, in principle. However, in general, the Rust compiler does not
 * guarantee codegen will be performed for a non-inline function either.
 * Therefore, this file exports all the helpers. In the future, this may be
 * revisited to reduce the number of exports after the compiler is informed
 * about the places codegen is required.
 *
 * All symbols are exported as GPL-only to guarantee no GPL-only feature is
 * accidentally exposed.
 *
 * Sorted alphabetically.
 */

#include <kunit/test-bug.h>
#include <linux/bug.h>
#include <linux/build_bug.h>
#include <linux/device.h>
#include <linux/err.h>
#include <linux/errname.h>
#include <linux/gfp.h>
#include <linux/highmem.h>
#include <linux/mutex.h>
#include <linux/refcount.h>
#include <linux/sched/signal.h>
#include <linux/slab.h>
#include <linux/spinlock.h>
#include <linux/wait.h>
#include <linux/workqueue.h>
#include <linux/mentor.h>
#include <linux/list.h>
#include <linux/i2c.h>
#include <linux/rwlock.h>

__noreturn void rust_helper_BUG(void)
{
	BUG();
}
EXPORT_SYMBOL_GPL(rust_helper_BUG);

unsigned long rust_helper_copy_from_user(void *to, const void __user *from,
					 unsigned long n)
{
	return copy_from_user(to, from, n);
}
EXPORT_SYMBOL_GPL(rust_helper_copy_from_user);

unsigned long rust_helper_copy_to_user(void __user *to, const void *from,
				       unsigned long n)
{
	return copy_to_user(to, from, n);
}
EXPORT_SYMBOL_GPL(rust_helper_copy_to_user);

void rust_helper_mutex_lock(struct mutex *lock)
{
	mutex_lock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_mutex_lock);

void rust_helper___spin_lock_init(spinlock_t *lock, const char *name,
				  struct lock_class_key *key)
{
#ifdef CONFIG_DEBUG_SPINLOCK
	__raw_spin_lock_init(spinlock_check(lock), name, key, LD_WAIT_CONFIG);
#else
	spin_lock_init(lock);
#endif
}
EXPORT_SYMBOL_GPL(rust_helper___spin_lock_init);

void rust_helper_spin_lock(spinlock_t *lock)
{
	spin_lock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_spin_lock);

void rust_helper_spin_unlock(spinlock_t *lock)
{
	spin_unlock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_spin_unlock);

void rust_helper_init_wait(struct wait_queue_entry *wq_entry)
{
	init_wait(wq_entry);
}
EXPORT_SYMBOL_GPL(rust_helper_init_wait);

int rust_helper_signal_pending(struct task_struct *t)
{
	return signal_pending(t);
}
EXPORT_SYMBOL_GPL(rust_helper_signal_pending);

struct page *rust_helper_alloc_pages(gfp_t gfp_mask, unsigned int order)
{
	return alloc_pages(gfp_mask, order);
}
EXPORT_SYMBOL_GPL(rust_helper_alloc_pages);

void *rust_helper_kmap_local_page(struct page *page)
{
	return kmap_local_page(page);
}
EXPORT_SYMBOL_GPL(rust_helper_kmap_local_page);

void rust_helper_kunmap_local(const void *addr)
{
	kunmap_local(addr);
}
EXPORT_SYMBOL_GPL(rust_helper_kunmap_local);

refcount_t rust_helper_REFCOUNT_INIT(int n)
{
	return (refcount_t)REFCOUNT_INIT(n);
}
EXPORT_SYMBOL_GPL(rust_helper_REFCOUNT_INIT);

void rust_helper_refcount_inc(refcount_t *r)
{
	refcount_inc(r);
}
EXPORT_SYMBOL_GPL(rust_helper_refcount_inc);

bool rust_helper_refcount_dec_and_test(refcount_t *r)
{
	return refcount_dec_and_test(r);
}
EXPORT_SYMBOL_GPL(rust_helper_refcount_dec_and_test);

__force void *rust_helper_ERR_PTR(long err)
{
	return ERR_PTR(err);
}
EXPORT_SYMBOL_GPL(rust_helper_ERR_PTR);

bool rust_helper_IS_ERR(__force const void *ptr)
{
	return IS_ERR(ptr);
}
EXPORT_SYMBOL_GPL(rust_helper_IS_ERR);

long rust_helper_PTR_ERR(__force const void *ptr)
{
	return PTR_ERR(ptr);
}
EXPORT_SYMBOL_GPL(rust_helper_PTR_ERR);

const char *rust_helper_errname(int err)
{
	return errname(err);
}
EXPORT_SYMBOL_GPL(rust_helper_errname);

struct task_struct *rust_helper_get_current(void)
{
	return current;
}
EXPORT_SYMBOL_GPL(rust_helper_get_current);

void rust_helper_get_task_struct(struct task_struct *t)
{
	get_task_struct(t);
}
EXPORT_SYMBOL_GPL(rust_helper_get_task_struct);

void rust_helper_put_task_struct(struct task_struct *t)
{
	put_task_struct(t);
}
EXPORT_SYMBOL_GPL(rust_helper_put_task_struct);

struct kunit *rust_helper_kunit_get_current_test(void)
{
	return kunit_get_current_test();
}
EXPORT_SYMBOL_GPL(rust_helper_kunit_get_current_test);

void rust_helper_init_work_with_key(struct work_struct *work, work_func_t func,
				    bool onstack, const char *name,
				    struct lock_class_key *key)
{
	__init_work(work, onstack);
	work->data = (atomic_long_t)WORK_DATA_INIT();
	lockdep_init_map(&work->lockdep_map, name, key, 0);
	INIT_LIST_HEAD(&work->entry);
	work->func = func;
}
EXPORT_SYMBOL_GPL(rust_helper_init_work_with_key);

void * __must_check __realloc_size(2)
rust_helper_krealloc(const void *objp, size_t new_size, gfp_t flags)
{
	return krealloc(objp, new_size, flags);
}
EXPORT_SYMBOL_GPL(rust_helper_krealloc);

u32 rust_helper_mentor_read(u8 addr)
{
	return mentor_read(addr);
}
EXPORT_SYMBOL_GPL(rust_helper_mentor_read);

//------------ START HELPERS FOR LIST.H -----------------

void rust_helper_init_list_head(struct list_head *list) {
    INIT_LIST_HEAD(list);
}
EXPORT_SYMBOL_GPL(rust_helper_init_list_head);

void rust_helper_list_add(struct list_head *new_node, struct list_head *head) {
    list_add(new_node, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_add);

void rust_helper_list_add_tail(struct list_head *new_node, struct list_head *head) {
    list_add_tail(new_node, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_add_tail);

void rust_helper_list_del(struct list_head *entry) {
    list_del(entry);
}
EXPORT_SYMBOL_GPL(rust_helper_list_del);

void rust_helper_list_replace(struct list_head *old, struct list_head *new_node) {
    list_replace(old, new_node);
}
EXPORT_SYMBOL_GPL(rust_helper_list_replace);

void rust_helper_list_replace_init(struct list_head *old, struct list_head *new_node) {
    list_replace_init(old, new_node);
}
EXPORT_SYMBOL_GPL(rust_helper_list_replace_init);

void rust_helper_list_move(struct list_head *list, struct list_head *head) {
    list_move(list, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_move);

void rust_helper_list_move_tail(struct list_head *list, struct list_head *head) {
    list_move_tail(list, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_move_tail);

int rust_helper_list_empty(struct list_head *head) {
    return list_empty(head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_empty);

void rust_helper_list_splice(struct list_head *list, struct list_head *head) {
    list_splice(list, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_splice);

void rust_helper_list_splice_init(struct list_head *list, struct list_head *head) {
    list_splice_init(list, head);
}
EXPORT_SYMBOL_GPL(rust_helper_list_splice_init);

//------------ END HELPERS FOR LIST.H -----------------

//------------ START HELPERS FOR RWLOCK.H -----------------
void rust_helper_rwlock_init(rwlock_t *lock, const char *name, struct lock_class_key *key)
{
    rwlock_init(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_rwlock_init);

void rust_helper_read_lock(rwlock_t *lock)
{
    read_lock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_read_lock);

void rust_helper_read_unlock(rwlock_t *lock)
{
    read_unlock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_read_unlock);

void rust_helper_write_lock(rwlock_t *lock)
{
    write_lock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_write_lock);

void rust_helper_write_unlock(rwlock_t *lock)
{
    write_unlock(lock);
}
EXPORT_SYMBOL_GPL(rust_helper_write_unlock);

//------------ END HELPERS FOR RWLOCK.H -----------------

//------------ START HELPERS FOR I2C.H -----------------
void rust_helper_i2c_add_adapter(struct i2c_adapter *adapter)
{
    i2c_add_adapter(adapter);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_add_adapter);

void rust_helper_i2c_del_adapter(struct i2c_adapter *adapter)
{
    i2c_del_adapter(adapter);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_del_adapter);

s32 rust_helper_i2c_smbus_read_byte_data(struct i2c_client *client, u8 command)
{
    return i2c_smbus_read_byte_data(client, command);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_smbus_read_byte_data);

s32 rust_helper_i2c_smbus_write_byte_data(struct i2c_client *client, u8 command, u8 value)
{
    return i2c_smbus_write_byte_data(client, command, value);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_smbus_write_byte_data);

int rust_helper_i2c_register_driver(struct module *module, struct i2c_driver *driver)
{
    return i2c_register_driver(module, driver);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_register_driver);

void rust_helper_i2c_del_driver(struct i2c_driver *driver)
{
    i2c_del_driver(driver);
}
EXPORT_SYMBOL_GPL(rust_helper_i2c_del_driver);

//------------ END HELPERS FOR I2C.H -----------------


/*
 * `bindgen` binds the C `size_t` type as the Rust `usize` type, so we can
 * use it in contexts where Rust expects a `usize` like slice (array) indices.
 * `usize` is defined to be the same as C's `uintptr_t` type (can hold any
 * pointer) but not necessarily the same as `size_t` (can hold the size of any
 * single object). Most modern platforms use the same concrete integer type for
 * both of them, but in case we find ourselves on a platform where
 * that's not true, fail early instead of risking ABI or
 * integer-overflow issues.
 *
 * If your platform fails this assertion, it means that you are in
 * danger of integer-overflow bugs (even if you attempt to add
 * `--no-size_t-is-usize`). It may be easiest to change the kernel ABI on
 * your platform such that `size_t` matches `uintptr_t` (i.e., to increase
 * `size_t`, because `uintptr_t` has to be at least as big as `size_t`).
 */
static_assert(
	sizeof(size_t) == sizeof(uintptr_t) &&
	__alignof__(size_t) == __alignof__(uintptr_t),
	"Rust code expects C `size_t` to match Rust `usize`"
);

// This will soon be moved to a separate file, so no need to merge with above.
#include <linux/blk-mq.h>
#include <linux/blkdev.h>

void *rust_helper_blk_mq_rq_to_pdu(struct request *rq)
{
	return blk_mq_rq_to_pdu(rq);
}
EXPORT_SYMBOL_GPL(rust_helper_blk_mq_rq_to_pdu);

struct request *rust_helper_blk_mq_rq_from_pdu(void *pdu)
{
	return blk_mq_rq_from_pdu(pdu);
}
EXPORT_SYMBOL_GPL(rust_helper_blk_mq_rq_from_pdu);
