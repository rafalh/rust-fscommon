#[cfg(not(feature = "std"))]
extern crate core;

#[cfg(not(feature = "std"))]
extern crate core_io;

#[macro_use]
extern crate log;

#[cfg(not(feature = "std"))]
use core_io as io;

#[cfg(feature = "std")]
use std as core;
#[cfg(feature = "std")]
use std::io as io;

mod buf_stream;
mod stream_slice;

pub use buf_stream::*;
pub use stream_slice::*;
