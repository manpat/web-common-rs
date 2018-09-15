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

	pub fn enable_color_write(&self) {
		unsafe { gl::ColorMask(1, 1, 1, 1) }
	}
	pub fn disable_color_write(&self) {
		unsafe { gl::ColorMask(0, 0, 0, 0) }
	}

	pub fn enable_depth_write(&self) {
		unsafe { gl::DepthMask(1) }
	}
	pub fn disable_depth_write(&self) {
		unsafe { gl::DepthMask(0) }
	}

	pub fn set_stencil(&self, params: StencilParams) {
		unsafe {
			gl::StencilFunc(params.condition, params.reference as i32, 0xff);
			gl::StencilOp(params.stencil_fail, params.depth_fail, params.pass);
		}
	}
}

pub struct StencilParams {
	pub condition: u32,
	pub reference: u8,

	pub stencil_fail: u32,
	pub depth_fail: u32,
	pub pass: u32,
}

impl StencilParams {
	pub fn new(reference: u8) -> Self {
		StencilParams {
			reference,
			condition: gl::NEVER,

			stencil_fail: gl::KEEP,
			depth_fail: gl::KEEP,
			pass: gl::KEEP,
		}
	}


	pub fn pass_if(self, condition: u32) -> Self {
		StencilParams { condition, ..self }
	}

	pub fn always(self) -> Self { self.pass_if(gl::ALWAYS) }
	pub fn never(self) -> Self { self.pass_if(gl::NEVER) }
	pub fn equal(self) -> Self { self.pass_if(gl::EQUAL) }
	pub fn less_than_stencil(self) -> Self { self.pass_if(gl::LESS) }
	pub fn greater_than_stencil(self) -> Self { self.pass_if(gl::GREATER) }

	pub fn replace(self) -> Self {
		Self { pass: gl::REPLACE, ..self }
	}

	pub fn increment(self) -> Self {
		Self { pass: gl::INCR, ..self }
	}

	pub fn decrement(self) -> Self {
		Self { pass: gl::DECR, ..self }
	}

	pub fn invert(self) -> Self {
		Self { pass: gl::INVERT, ..self }
	}
}