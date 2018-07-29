use bindings::emscripten::*;
use bindings::gl;

use common::color::*;
use common::vector::Vec2i;

pub struct WebGLContext {
	ems_context: EMSCRIPTEN_WEBGL_CONTEXT_HANDLE,
}

impl WebGLContext {
	pub fn new(alpha: bool) -> Self {
		use std::mem::uninitialized;

		let ems_context_handle = unsafe {
			let mut attribs = uninitialized();
			emscripten_webgl_init_context_attributes(&mut attribs);
			attribs.alpha = if alpha {1} else {0};
			attribs.stencil = 1;
			attribs.antialias = 1;
			attribs.preserveDrawingBuffer = 0;
			attribs.enableExtensionsByDefault = 0;

			emscripten_webgl_create_context(b"canvas\0".as_ptr() as _, &attribs)
		};

		match ems_context_handle {
			EMSCRIPTEN_RESULT_NOT_SUPPORTED => {
				panic!("WebGL not supported");
			}

			EMSCRIPTEN_RESULT_FAILED_NOT_DEFERRED => {
				panic!("WebGL context creation failed (FAILED_NOT_DEFERRED)");
			}

			EMSCRIPTEN_RESULT_FAILED => {
				panic!("WebGL context creation failed (FAILED)");
			}

			x if x < 0 => {
				panic!("WebGL context creation failed ({})", x);
			}

			_ => {}
		}

		if unsafe {emscripten_webgl_make_context_current(ems_context_handle) != EMSCRIPTEN_RESULT_SUCCESS} {
			panic!("Failed to make webgl context current");
		}

		WebGLContext { ems_context: ems_context_handle }
	}
	
	pub fn clear_color(&self) {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}
	}

	pub fn clear_all(&self) {
		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
		}
	}

	pub fn set_background<C>(&self, col: C) where C: Into<Color> {
		unsafe {
			let c = col.into();
			gl::ClearColor(c.r, c.g, c.b, c.a);
		}
	}

	pub fn set_viewport<V>(&self, size: V) where V: Into<Vec2i> {
		unsafe {
			let s = size.into();
			gl::Viewport(0, 0, s.x, s.y);
		}
	}
}