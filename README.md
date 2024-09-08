# Rust4Linux
This repository contains all the training examples and some trash code that i will produce.

# Rust4LinuxExperiments
Welcome to the Rust for Linux Kernel Experiments repository. <br />
This project contains experiments focused on exploring the support for the Rust programming language within the Linux kernel. Specifically, we will be creating out-of-tree kernel modules to investigate the extent to which Rust can be integrated and utilized in kernel development.

## Introduction
Rust is a modern programming language that promises memory safety and concurrency without sacrificing performance. These features make it an attractive candidate for systems programming, including operating system kernels. This repository contains a series of experimental out-of-tree modules for the Linux kernel written in Rust, designed to test the capabilities and limitations of Rust in this context.

## Project Goals
-   Explore Rust Integration: Investigate the integration of Rust with the Linux kernel.
-   Develop Kernel Modules: Create and test out-of-tree kernel modules written in Rust.
-   Document Findings: Share the results, challenges, and solutions encountered during the experiments.
-   Contribute to the Community: Provide valuable insights and potential contributions to the Linux and Rust communities.

## Current Progess
### Kprobes Module
This module demonstrates how Rust can interact with C code to register Kprobes within a Rust-based kernel module. Kprobes is a powerful debugging mechanism in the Linux kernel, allowing you to dynamically break into any kernel routine and collect debugging and performance information non-disruptively.

**Features of the Kprobes Module**
- Interaction with C Code: The module interacts with existing C code to leverage Kprobe functionality.
- Registration of Kprobes: It allows for the registration and management of Kprobes from within a Rust module.

### CharDevice
This Rust-based Linux kernel module provides a character device that allows user-space programs to manage access control rules associated with specific user IDs (uid). <br />
The device leverages IOCTL (Input/Output Control) commands to add, remove, and retrieve rules from a global in-kernel data store (USER_RULE_STORE). The module contains also CLI (sec_tools.c) that can be used by the user.

**Functionalities**
- *Add*: Adds an access control rule for a specific user.
- *Remove*: Removes an access control rule for a specific user.
- *Read*: Retrieves all access control rules for a given user, or all users, if no specific uid is provided.

### Mentor: Creating Bindings to a simple driver

We added a simple dummy device driver named `mentor` to the Linux kernel and created Rust bindings for its functions.

- **Driver Overview:**
  - The `mentor` driver implements basic read and write operations on a dummy device.
  - The driver uses a spinlock (`mentor_lock`) to manage concurrent access to a shared data structure (`mentor_data`).
  - It also includes a simulated undefined behavior function for testing purposes.

- **Exported Functions:**
  - `mentor_write(u8 addr, u32 value)`: Writes a value to the specified address in the dummy device.
  - `__mentor_read(u8 addr) -> u32`: Reads a value from the specified address in the dummy device.

- **Creating Rust Bindings:**
  - We created Rust bindings for the exported `mentor` functions by writing corresponding Rust code that interacts with these functions through the FFI (Foreign Function Interface).
  - The bindings allow us to call the `mentor_write` and `__mentor_read` functions directly from Rust code.

- **Testing:**
  - We wrote a test module in Rust that utilizes the `mentor` functions to verify the correctness of the bindings.
  - Ensured that the module compiled correctly and integrated with the Linux kernel.


### Jiffies
We created Rust bindings for the `jiffies` kernel functions, which provide a way to interact with the Linux kernel's timekeeping mechanism. The `jiffies` counter is a global variable that tracks the number of timer interrupts that have occurred since the system booted.

- **Functions Bound:**
  - `jiffies_to_msecs(j)`: Converts a given jiffies value to milliseconds.
  - `jiffies_to_usecs(j)`: Converts a given jiffies value to microseconds.
  
- **Safety Measures:**
  - Ensured the `j` parameter for `jiffies_to_usecs` is of type `u64` to maintain type safety and prevent overflows.

- **Implementation:**
  - We wrote a Rust abstraction in `jiffies.rs` to call these functions safely from Rust code.
  - This required including the relevant kernel header and writing the Rust bindings to map to the C functions.

