#![feature(generators, generator_trait, box_syntax)]
#![feature(inclusive_range_syntax)]
#![feature(specialization)]
#![feature(ord_max_min)]
#![feature(link_args)]
#![feature(const_fn)]

pub extern crate common;
pub use common::*;

#[macro_use] pub mod bindings;
#[macro_use] pub mod coro_util;

pub mod rendering;
pub mod events;
pub mod webgl;

pub mod paper;

pub use bindings::emscripten::*;
pub use coro_util::*;
pub use webgl::*;

pub use paper::*;

pub use rendering::gl;
pub use rendering::shader::*;