#![no_std]
#![feature(binary_heap_into_iter_sorted)]

extern crate acid_io;
extern crate alloc;
#[cfg(feature = "std")]
extern crate difference;
extern crate hashbrown;
#[cfg(feature = "std")]
extern crate std;

#[cfg(not(feature = "std"))]
use acid_io as io;
#[cfg(feature = "std")]
use std::io;

pub mod edge;
pub mod error;
pub mod graph;
pub mod node;
pub mod result;
