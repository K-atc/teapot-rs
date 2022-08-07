#![no_std]
#![feature(binary_heap_into_iter_sorted)]

extern crate acid_io;
extern crate alloc;
#[cfg(feature = "std")]
extern crate difference;
extern crate hashbrown;
#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
extern crate log;

#[cfg(not(feature = "std"))]
use acid_io as io;
#[cfg(feature = "std")]
use std::io;

pub mod edge;
pub mod error;
pub mod graph;
pub mod node;
pub mod result;

#[macro_export]
macro_rules! metrics {
    ( $ident:ident ) => {{
        #[cfg(feature = "metrics")]
        $ident
    }};
    ( $expr:expr ) => {{
        #[cfg(feature = "metrics")]
        $expr
    }};
    ( $expr:expr; ) => {{
        #[cfg(feature = "metrics")]
        $expr;
    }};
    ( $stmt:stmt ) => {{
        #[cfg(feature = "metrics")]
        $stmt
    }};
    ( $block:block ) => {{
        #[cfg(feature = "metrics")]
        $block
    }};
}
