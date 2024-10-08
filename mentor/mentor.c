// SPDX-License-Identifier: GPL-2.0

#include <linux/mentor.h>
#include <linux/spinlock.h>
#include <linux/module.h>
#include <linux/kernel.h>

static DEFINE_SPINLOCK(mentor_lock);
static u32 mentor_data[MENTOR_TOTAL_WRITES_ADDR + 1] = { 40, 41, 42, 43, 44, 0 };

static u32 mentor_simulate_undefined_behavior(void) {
	printk(KERN_CRIT "mentor: undefined behavior!\n");
	return 0xFFFFFFFF;
}

u32 __mentor_read(u8 addr)
{
	u32 result;
	unsigned long flags;

	if (addr > MENTOR_TOTAL_WRITES_ADDR)
		return mentor_simulate_undefined_behavior();

	spin_lock_irqsave(&mentor_lock, flags);
	result = mentor_data[addr];
	spin_unlock_irqrestore(&mentor_lock, flags);

	return result;
}
EXPORT_SYMBOL_GPL(__mentor_read);

void mentor_write(u8 addr, u32 value)
{
	unsigned long flags;

	if (addr >= MENTOR_TOTAL_WRITES_ADDR) {
		mentor_simulate_undefined_behavior();
		return;
	}

	spin_lock_irqsave(&mentor_lock, flags);
	mentor_data[addr] = value;
	++mentor_data[MENTOR_TOTAL_WRITES_ADDR];
	spin_unlock_irqrestore(&mentor_lock, flags);
}
EXPORT_SYMBOL_GPL(mentor_write);


static int __init my_init(void)
{
	printk(KERN_INFO "mentor: Hello from Mentor module.");
	return 0;
}

static void __exit my_exit(void)
{
	printk(KERN_INFO "mentor: Bye from Mentor module.");
}

module_init(my_init);
module_exit(my_exit);

MODULE_LICENSE("GPL");
MODULE_AUTHOR("Luca Saverio Esposito");
MODULE_DESCRIPTION("Test for Rust bindings");
