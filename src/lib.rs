#![allow(dead_code)]
#![feature(lang_items)]

#![feature(test)]
extern crate test;

extern crate stopwatch;
//extern crate jemallocator;
//#[global_allocator]
//static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;


#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate arrayvec;
//extern crate dlopen;
//#[macro_use]
//extern crate dlopen_derive;
//#[macro_use]
//extern crate lazy_static;
extern crate libc;
extern crate time;
extern crate byteorder;

#[macro_use]
pub mod macros;

pub mod error;
#[cfg_attr(feature = "cargo-clippy",
allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod sds;
#[cfg_attr(feature = "cargo-clippy",
allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod listpack;
#[cfg_attr(feature = "cargo-clippy",
allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod rax;
#[cfg_attr(feature = "cargo-clippy",
allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod stream;
#[macro_use]
pub mod key;

#[cfg_attr(feature = "cargo-clippy",
allow(redundant_field_names, suspicious_arithmetic_impl))]
pub mod redis;
pub mod mod_api;
pub mod sliced;

