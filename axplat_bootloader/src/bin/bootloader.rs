//! Standalone bootloader binary

#![no_std]
#![no_main]

// Use the bootloader library
use axplat_bootloader;

// The bootloader entry point is already defined in the library
// This file just exists to create a binary target
