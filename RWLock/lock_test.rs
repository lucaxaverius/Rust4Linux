// SPDX-License-Identifier: GPL-2.0

//! Test module for RwLock, Mutex, and Spinlock.

use kernel::{
    prelude::*,
    sync::{
        new_mutex,
        new_rwlock,
        new_spinlock,
        Arc,
        Mutex,
        RwLock,
        SpinLock,
    },
    workqueue::{self, impl_has_work, new_work, Work, WorkItem},
};

module! {
    type: LockTestModule,
    name: "lock_test_module",
    author: "Luca Saverio Esposito",
    description: "Test module for RwLock, Mutex, and Spinlock",
    license: "GPL",
}

struct SharedData {
    value: u32,
}

struct LockTestModule {
    rwlock: Arc<Pin<Box<RwLock<SharedData>>>>,
    mutex: Arc<Pin<Box<Mutex<SharedData>>>>,
    spinlock: Arc<Pin<Box<SpinLock<SharedData>>>>,
}

impl kernel::Module for LockTestModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("LockTestModule init\n");

        // Initialize the shared data structures and wrap them in Arc
        let rwlock = Arc::new(Box::pin_init(
            new_rwlock!(SharedData { value: 1 }), GFP_KERNEL
        )?,GFP_KERNEL)?;

        let mutex = Arc::new(Box::pin_init(
            new_mutex!(SharedData { value: 2 }), GFP_KERNEL
        )?,GFP_KERNEL)?;

        let spinlock = Arc::new(Box::pin_init(
            new_spinlock!(SharedData { value: 3 }), GFP_KERNEL
        )?,GFP_KERNEL)?;

        let module = Self {
            rwlock,
            mutex,
            spinlock,
        };

        // Start test threads
        module.test_rwlock()?;
        //module.test_mutex()?;
        //module.test_spinlock()?;

        Ok(module)
    }
}

impl LockTestModule {
    fn test_rwlock(&self) -> Result {
        pr_info!("Testing RwLock\n");

        // Clone the Arc to pass to worker structs
        let rwlock = self.rwlock.clone();

        // Create reader work items
        for i in 0..16 {
            let work_item = RwLockReaderWork::new(rwlock.clone(), i)?;
            let _ = workqueue::system_unbound().enqueue(work_item); 
        }

        // Create writer work item with a simulated initial delay
        let work_item = RwLockWriterWork::new(rwlock.clone(), /* initial_delay */ 5000000)?;
        let _ = workqueue::system_unbound().enqueue(work_item); 
         
        Ok(())
    }

    fn test_mutex(&self) -> Result {
        pr_info!("Testing Mutex\n");

        let mutex = self.mutex.clone();

        // Create mutex work items
        for i in 0..3 {
            let work_item = MutexWork::new(mutex.clone(), i)?;
            let _ = workqueue::system_unbound().enqueue(work_item); 
             
        }

        Ok(())
    }

    fn test_spinlock(&self) -> Result {
        pr_info!("Testing Spinlock\n");

        let spinlock = self.spinlock.clone();

        // Create spinlock work items
        for i in 0..3 {
            let work_item = SpinLockWork::new(spinlock.clone(), i)?;
            let _ = workqueue::system_unbound().enqueue(work_item);
        }

        Ok(())
    }
}

impl Drop for LockTestModule {
    fn drop(&mut self) {
        pr_info!("LockTestModule exit\n");
    }
}

/// This function does dummy operation to simulate a delay
pub fn simulate_delay(delay: u32) {
    let mut a = 1;
    for _ in 0..delay {
        a = a + 1;
    }
}

#[pin_data]
struct RwLockReaderWork {
    reader_id: usize,
    #[pin]
    work: Work<RwLockReaderWork>,
    rwlock: Arc<Pin<Box<RwLock<SharedData>>>>,
}

impl_has_work! {
    impl HasWork<Self> for RwLockReaderWork { self.work }
}

impl RwLockReaderWork {
    fn new(
        rwlock: Arc<Pin<Box<RwLock<SharedData>>>>,
        reader_id: usize,
    ) -> Result<Arc<Self>> {
        Arc::pin_init(
            pin_init!(RwLockReaderWork {
                reader_id,
                rwlock,
                work <- new_work!("RwLockReaderWork::work"),
            }),
            GFP_KERNEL,
        )
    }
}

impl WorkItem for RwLockReaderWork {
    type Pointer = Arc<Self>;

    fn run(this: Arc<Self>) {
        pr_info!(
            "Reader {} attempting to acquire read lock\n",
            this.reader_id
        );
        let guard = this.rwlock.read_lock();
        pr_info!("Reader {} acquired read lock\n", this.reader_id);
        // Simulate some read operation
        let value = guard.value;
        pr_info!("Reader {} read value: {}\n", this.reader_id, value);
        // Simulate delay
        simulate_delay(1000000000);
        pr_info!("Reader {} releasing read lock\n", this.reader_id);
        // Guard is dropped here
    }
}

#[pin_data]
struct RwLockWriterWork {
    #[pin]
    work: Work<RwLockWriterWork>,
    rwlock: Arc<Pin<Box<RwLock<SharedData>>>>,
    initial_delay: u32,
}

impl_has_work! {
    impl HasWork<Self> for RwLockWriterWork { self.work }
}

impl RwLockWriterWork {
    fn new(
        rwlock: Arc<Pin<Box<RwLock<SharedData>>>>,
        initial_delay: u32,
    ) -> Result<Arc<Self>> {
        Arc::pin_init(
            pin_init!(RwLockWriterWork {
                rwlock,
                initial_delay,
                work <- new_work!("RwLockWriterWork::work"),
            }),
            GFP_KERNEL,
        )
    }
}

impl WorkItem for RwLockWriterWork {
    type Pointer = Arc<Self>;

    fn run(this: Arc<Self>) {
        // Simulate initial delay
        simulate_delay(this.initial_delay);

        pr_info!("Writer attempting to acquire write lock\n");
        let mut guard = this.rwlock.lock();
        pr_info!("Writer acquired write lock\n");
        // Simulate a write operation
        guard.value += 1;
        pr_info!("Writer incremented value to: {}\n", guard.value);
        // Simulate delay
        simulate_delay(1000);
        pr_info!("Writer releasing write lock\n");
        // Guard is dropped here
    }
}

#[pin_data]
struct MutexWork {
    thread_id: usize,
    #[pin]
    work: Work<MutexWork>,
    mutex: Arc<Pin<Box<Mutex<SharedData>>>>,
}

impl_has_work! {
    impl HasWork<Self> for MutexWork { self.work }
}

impl MutexWork {
    fn new(
        mutex: Arc<Pin<Box<Mutex<SharedData>>>>,
        thread_id: usize,
    ) -> Result<Arc<Self>> {
        Arc::pin_init(
            pin_init!(MutexWork {
                thread_id,
                mutex,
                work <- new_work!("MutexWork::work"),
            }),
            GFP_KERNEL,
        )
    }
}

impl WorkItem for MutexWork {
    type Pointer = Arc<Self>;

    fn run(this: Arc<Self>) {
        pr_info!(
            "Thread {} attempting to acquire mutex\n",
            this.thread_id
        );
        let mut guard = this.mutex.lock();
        pr_info!("Thread {} acquired mutex\n", this.thread_id);
        // Simulate some operation
        guard.value += 1;
        pr_info!(
            "Thread {} incremented value to: {}\n",
            this.thread_id,
            guard.value
        );
        // Simulate delay
        simulate_delay(500);
        pr_info!("Thread {} releasing mutex\n", this.thread_id);
        // Guard is dropped here
    }
}

#[pin_data]
struct SpinLockWork {
    thread_id: usize,
    #[pin]
    work: Work<SpinLockWork>,
    spinlock: Arc<Pin<Box<SpinLock<SharedData>>>>,
}

impl_has_work! {
    impl HasWork<Self> for SpinLockWork { self.work }
}

impl SpinLockWork {
    fn new(
        spinlock: Arc<Pin<Box<SpinLock<SharedData>>>>,
        thread_id: usize,
    ) -> Result<Arc<Self>> {
        Arc::pin_init(
            pin_init!(SpinLockWork {
                thread_id,
                spinlock,
                work <- new_work!("SpinLockWork::work"),
            }),
            GFP_KERNEL,
        )
    }
}

impl WorkItem for SpinLockWork {
    type Pointer = Arc<Self>;

    fn run(this: Arc<Self>) {
        pr_info!(
            "Thread {} attempting to acquire spinlock\n",
            this.thread_id
        );
        let mut guard = this.spinlock.lock();
        pr_info!("Thread {} acquired spinlock\n", this.thread_id);
        // Simulate some operation
        guard.value += 1;
        pr_info!(
            "Thread {} incremented value to: {}\n",
            this.thread_id,
            guard.value
        );
        // Simulate delay
        simulate_delay(250);
        pr_info!("Thread {} releasing spinlock\n", this.thread_id);
        // Guard is dropped here
    }
}
