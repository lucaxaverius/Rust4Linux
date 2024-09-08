# Guida all'installazione
Per poter installare il modulo mentor_test è necessario ricompilare il kernel aggiungendo il device mentor.c.

## linux/drivers
All'interno di kernel/linux/drivers bisogna aggiungere il driver. Queste due modifiche permettono di aggiungere al processo di build il nuovo dispositivo mentor.

1. **In Kconfig** 
    ```bash
    source "drivers/mentor/Kconfig"

2. **Makefile** 
    ```bash
    obj-$(CONFIG_MENTOR) += mentor/


## linux/drivers/mentor
Questa directory conterrà il codice del driver e del modulo Rust di test utilizzato per verificare il funzionamento dei bindings.

1. **Kconfig**
    ```bash
        # SPDX-License-Identifier: GPL-2.0
        menuconfig MENTOR
            bool "Mentor Support"
            help
            This enables Mentor support.

            If unsure, say N.

        if MENTOR

        config MENTOR_TEST
            tristate "Mentor test module"
            help
            Say Y or M here to build support for the Mentor test module.

            If driver is built as a module it will be called mentor_test.

            If unsure, say N.
        endif # MENTOR 

2. **Makefile**
    ```bash
        obj-$(CONFIG_MENTOR)		+= mentor.o
        obj-$(CONFIG_MENTOR_TEST)	+= mentor_test.o

3. **mentor.c**
    ```bash
        // SPDX-License-Identifier: GPL-2.0

        #include <linux/mentor.h>
        #include <linux/spinlock.h>

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

4. **mentor_test.rs**
    ```bash
        // SPDX-License-Identifier: GPL-2.0

        //! Mentor test

        #![no_std]
        #![feature(allocator_api, global_asm)]

        use kernel::{mentor, prelude::*, str::CStr, ThisModule};

        module! {
            type: MentorTest,
            name: b"mentor_test",
            author: b"Rust for Linux Contributors",
            description: b"Mentor Test",
            license: b"GPL v2",
            params: {
                write_addr: u8 {
                    default: 0,
                    permissions: 0,
                    description: b"Address to write",
                },
                write_value: u32 {
                    default: 42,
                    permissions: 0,
                    description: b"Value to write",
                },
            },
        }

        struct MentorTest;

        impl KernelModule for MentorTest {
            fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
                // Read module parameters
                let addr = *write_addr.read();
                let value = *write_value.read();

                // Never use bindings directly! Always create a safe abstraction.
                // This will likely be enforced in the future. Shown only
                // for demonstration purposes.
                {
                    use kernel::bindings;

                    pr_info!("--- Without an abstraction (do not use!)\n");

                    pr_info!("Writing value {} to address {}\n", value, addr);
                    unsafe { bindings::mentor_write(addr, value) };

                    pr_info!("Reading from address {}\n", addr);
                    let value = unsafe { bindings::mentor_read(addr) };
                    pr_info!("Read value = {}\n", value);

                    let total_writes =
                        unsafe { bindings::mentor_read(bindings::MENTOR_TOTAL_WRITES_ADDR as u8) };
                    pr_info!("Total writes = {}\n", total_writes);

                    // We can produce undefined behavior, just like in C.
                    let bad_addr = 0x42;
                    pr_info!("Reading from address {}\n", bad_addr);
                    let _ = unsafe { bindings::mentor_read(bad_addr) };
                }

                // The proper way.
                {
                    pr_info!("--- With a safe abstraction\n");

                    pr_info!("Writing value {} to address {}\n", value, addr);
                    mentor::write(addr, value)?;

                    pr_info!("Reading from address {}\n", addr);
                    let value = mentor::read(addr)?;
                    pr_info!("Read value = {}\n", value);

                    let total_writes = mentor::read_total_writes();
                    pr_info!("Total writes = {}\n", total_writes);

                    // Whatever we try to do here, as long as it is safe code,
                    // we cannot produce UB.
                    let bad_addr = 0x42;
                    pr_info!("Reading from address {}\n", bad_addr);
                    if mentor::read(bad_addr).is_err() {
                        pr_info!("Expected failure\n");
                    }
                }

                Ok(MentorTest)
            }
        }


## linux/include/linux

Bisogna aggiungere l'header relativo al modulo, necessario come riferimento per la generazione dei bindings

1. **mentor.h**
    ```bash
        /* SPDX-License-Identifier: GPL-2.0 */
        /*
        * The example mentor subsystem: a key-value "database".
        *
        * Valid addresses go from 0x00 to 0x05. Accessing others is UB.
        *
        * Reading address 0x05 gives the total number of writes.
        * Writing to it is UB.
        */
        #ifndef __LINUX_MENTOR_H
        #define __LINUX_MENTOR_H

        #include <linux/compiler.h>

        #define MENTOR_TOTAL_WRITES_ADDR 0x05

        /* Public interface */
        #define mentor_read(addr) \
            __mentor_read(addr)
        void mentor_write(u8 addr, u32 value);

        /* Do not use! */
        u32 __mentor_read(u8 addr);

        #endif /* __LINUX_MENTOR_H */   

## linux/rust
Bisogna includere l'header del device e dichiarere un helper per le macro che non sono delle semplici #define.

1. **helpers.c**
    ```bash
        #include <linux/mentor.h>

        u32 rust_helper_mentor_read(u8 addr)
        {
            return mentor_read(addr);
        }
        EXPORT_SYMBOL_GPL(rust_helper_mentor_read)

## linux/rust/kernel
Qui vengono definete le astrazioni per il nuovo device.

1. **bindings_helper.h**
    ```bash
        #include <linux/mentor.h>
2. **lib.rs**
    ```bash
        pub mod mentor;
3. **mentor.rs**
    ```bash
        // SPDX-License-Identifier: GPL-2.0

        //! Mentor subsystem.
        //!
        //! C headers: [`include/linux/mentor.h`](../../../../include/linux/mentor.h)

        use crate::{bindings, error::Error, Result};

        const TOTAL_WRITES_ADDR: u8 = bindings::MENTOR_TOTAL_WRITES_ADDR as u8;

        fn is_valid(addr: u8) -> bool {
            addr < TOTAL_WRITES_ADDR
        }

        /// Reads from an address (unchecked version).
        ///
        /// To read the total number of writes, use [`read_total_writes`] instead.
        ///
        /// # Safety
        ///
        /// The address must be valid.
        ///
        /// # Examples
        ///
        /// ```
        /// # use kernel::prelude::*;
        /// # use kernel::mentor;
        /// # fn test() {
        /// let result = unsafe { mentor::read_unchecked(0x01) };
        /// # }
        /// ```
        pub unsafe fn read_unchecked(addr: u8) -> u32 {
            // SAFETY: FFI call, the caller guarantees the address is valid.
            unsafe { bindings::mentor_read(addr) }
        }

        /// Reads from an address.
        ///
        /// To read the total number of writes, use [`read_total_writes`] instead.
        ///
        /// Returns an error if the address is invalid.
        ///
        /// # Examples
        ///
        /// ```
        /// # use kernel::prelude::*;
        /// # use kernel::mentor;
        /// # fn test() -> Result {
        /// let result = mentor::read(0x01)?;
        /// # Ok(())
        /// # }
        /// ```
        pub fn read(addr: u8) -> Result<u32> {
            if !is_valid(addr) {
                return Err(Error::EINVAL);
            }

            // SAFETY: FFI call, we have verified the address is valid.
            Ok(unsafe { bindings::mentor_read(addr) })
        }

        /// Writes a value to an address (unchecked version).
        ///
        /// # Safety
        ///
        /// The address must be valid.
        ///
        /// # Examples
        ///
        /// ```
        /// # use kernel::prelude::*;
        /// # use kernel::mentor;
        /// # fn test() {
        /// unsafe { mentor::write_unchecked(0x01, 42); }
        /// # }
        /// ```
        pub unsafe fn write_unchecked(addr: u8, value: u32) {
            // SAFETY: FFI call, the caller guarantees the address is valid.
            unsafe { bindings::mentor_write(addr, value) }
        }

        /// Writes a value to an address.
        ///
        /// Returns an error if the address is invalid.
        ///
        /// # Examples
        ///
        /// ```
        /// # use kernel::prelude::*;
        /// # use kernel::mentor;
        /// # fn test() -> Result {
        /// mentor::write(0x01, 42)?;
        /// # Ok(())
        /// # }
        /// ```
        pub fn write(addr: u8, value: u32) -> Result {
            if !is_valid(addr) {
                return Err(Error::EINVAL);
            }

            // SAFETY: FFI call, we have verified the address is valid.
            unsafe { bindings::mentor_write(addr, value) }

            Ok(())
        }

        /// Reads the total number of writes (from the special Mentor address).
        ///
        /// # Examples
        ///
        /// ```
        /// # use kernel::prelude::*;
        /// # use kernel::mentor;
        /// # fn test() {
        /// let total_writes = mentor::read_total_writes();
        /// # }
        /// ```
        pub fn read_total_writes() -> u32 {
            // SAFETY: FFI call, this address is always valid.
            unsafe { bindings::mentor_read(TOTAL_WRITES_ADDR) }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_is_valid() {
                assert!(is_valid(0x00));
                assert!(is_valid(0x04));
                assert!(!is_valid(0x05));
            }
        }

Quest'ultimo file in particolare contiene le abstractions verso le funzionalità del driver, sono disponibili anche le funzionalità unchecked, quindi in cui non è garantita la safety per mostrare il funzionamento, dunque a scopo didattico. 

## Documentazione
Una volta modificati tutti i file, è possibile generare la nuova documentazione e verificare come sia stato aggiunto il supporto al device, all'interno della main directory del kernel tramite il comando:

```bash
    make LLVM=1 -j$(nproc) rustodc
```


## Installazione del driver
È necessario aprire il menù di configurazione ed abilitare il nuovo driver nell'apposita sezione
```bash
    make LLVM=1 menuconfig
```
Dopodiché una volta abilitato il nuovo dispositvo, bisogna buildare nuovamente il kernel:
```bash
    make LLVM=1 -j$(nproc)
```
Dopo installarlo e aggiornare il boot loader:
```bash
    sudo make install
    sudo update-grub
```
Fatto ciò, riavviando la macchina, il nuovo driver farà parte dell'immagine del kernel, le sue funzionalità saranno globalmente esportate (è possibile fericarlo tramite /proc/kallsysms). Dunque il modulo di test in rust potrà essere montato. 