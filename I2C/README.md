# README: Rust I2C Support and Simple I2C Test Driver

## Introduction

This README explains the process of adding I2C support in Rust within the Linux kernel and demonstrates a simple I2C test driver that interacts with the `i2c-stub` module. The driver is written in Rust and utilizes Rust bindings to the kernel's I2C subsystem.

## Contents

- [Introduction](#introduction)
- [Prerequisites](#prerequisites)
- [I2C Rust Library](#i2c-rust-library)
- [Simple I2C Test Driver](#simple-i2c-test-driver)
- [Loading and Testing the Driver](#loading-and-testing-the-driver)
- [Removing the Driver and Cleanup](#removing-the-driver-and-cleanup)
- [Summary](#summary)

## Prerequisites

- **Linux Kernel with Rust Support**: Ensure you have a Linux kernel that supports Rust (e.g., Linux 6.1 or later with Rust-for-Linux patches).
- **Rust Toolchain**: Install the Rust toolchain configured for kernel development.
- **Kernel Source Code**: Have the kernel source code with the Rust I2C bindings added.

## I2C Rust Library

The I2C Rust library provides Rust-friendly wrappers around the kernel's I2C APIs. It includes support for:

- **I2CAdapter**: Represents an I2C adapter.
- **I2CClient**: Represents an I2C client device.
- **I2CDriver**: Represents an I2C driver.

### Key Features

- Initialization and registration of I2C adapters and drivers.
- Reading from and writing to I2C device registers.
- Safe Rust abstractions over kernel I2C functions.

### Source Code

The library code is located in `i2c.rs` and includes implementations for the above structures with methods for common I2C operations.

## Simple I2C Test Driver

The simple I2C test driver demonstrates how to use the I2C Rust library to interact with an I2C device simulated by the `i2c-stub` module.

### What the Driver Does

- Registers an I2C driver with the kernel.
- Implements a `probe` function that is called when a matching I2C device is found.
- In the `probe` function:
  - Writes a byte (`0xAB`) to register `0x01` of the I2C device.
  - Reads back the byte from the same register and logs the value.

### Source Code

The driver code is located in `simple_i2c_driver.rs`.

## Loading and Testing the Driver
### 1. Load the `i2c-stub` Module
The `i2c-stub` module simulates an I2C bus with devices for testing purposes. <br />
It is located in `\linux\drivers\i2c`, to load it:
> sudo modprobe i2c-stub chip_addr=0x50 
>
This creates a virtual I2C bus with a device at address `0x50`.
### 2. Load the Simple I2C Test Driver
> sudo insmod i2c_test_driver.ko 
> 
### 3. Instantiate the I2C Device via Sysfs
Manually create the I2C device to trigger the driver's probe function.

- **Identify the I2C Bus Number**: Find the bus number assigned to the i2c-stub adapter:
    >  i2cdetect - l
    >

    - Look for an entry like `i2c-0 i2c SMBus stub driver`.

- **Instantiate the I2C Device**: Replace i2c-0 with your bus number if different:
    > echo rust_i2c_dev 0x50 | sudo tee /sys/bus/i2c/devices/i2c-1/new_device
    >
### 4. Verify Driver Operation

- **Check Kernel Messages**: Verify that the probe function was called and that the read/write operations succeeded:
    > dmesg | tail
    >

## Removing the Driver and Cleanup
### 1. Delete the I2C Device via Sysfs
Remove the I2C device to unbind it from the driver:
> echo 0x50 | sudo tee /sys/bus/i2c/devices/i2c-0/delete_device
>

- To verify:
    > ls /sys/bus/i2c/devices/
    >
    - The device 0-0050 should no longer be present.

### 2.  Unload the Simple I2C Test Driver Module
> sudo rmmod simple_i2c_driver
>

### 3. Unload the `i2c-stub` Module
If you no longer need the `i2c-stub` module:
> sudo rmmod i2c-stub 
>

## Summary
We have successfully
- Implemented Rust support for the I2C subsystem in the Linux kernel.
- Written a simple I2C test driver in Rust that interacts with a simulated I2C device.


