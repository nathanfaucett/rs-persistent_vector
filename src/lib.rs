#![feature(alloc)]
//#![no_std]
extern crate core;


extern crate alloc;

extern crate collection_traits;


mod persistent_vector;


pub use self::persistent_vector::PersistentVector;
