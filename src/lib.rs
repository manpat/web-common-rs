#![feature(coroutines, coroutine_trait)]
#![feature(specialization)]

pub extern crate common;
pub use common::*;

extern crate png;

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