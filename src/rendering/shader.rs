#![allow(dead_code)]

use std;
use math::*;
use rendering::gl;

#[derive(Copy, Clone)]
pub struct Shader {
	pub gl_handle: u32,

	pub proj_loc: i32,
	pub view_loc: i32,
}

impl Shader {
	pub fn new(vertex_shader_src: &str, fragment_shader_src: &str) -> Result<Shader, String> {
		use std::ffi::{CStr, CString};
		unsafe {
			let (vs,fs) = (gl::CreateShader(gl::VERTEX_SHADER), gl::CreateShader(gl::FRAGMENT_SHADER));
			let program = gl::CreateProgram();

			for &(sh, src) in [(vs, vertex_shader_src), (fs, fragment_shader_src)].iter() {
				let src = CString::new(src).unwrap();
				gl::ShaderSource(sh, 1, &src.as_ptr(), std::ptr::null());
				gl::CompileShader(sh);

				let mut status = 0i32;
				gl::GetShaderiv(sh, gl::COMPILE_STATUS, &mut status);
				if status == 0 {
					let mut buf = [0u8; 1024];
					let mut len = 0;
					gl::GetShaderInfoLog(sh, buf.len() as _, &mut len, buf.as_mut_ptr() as _);

					return Err(CStr::from_bytes_with_nul_unchecked(&buf[..len as usize]).to_string_lossy().into());
				}
				
				gl::AttachShader(program, sh);
			}

			gl::LinkProgram(program);

			gl::DeleteShader(vs);
			gl::DeleteShader(fs);

			Ok(Shader {
				gl_handle: program,

				proj_loc: gl::GetUniformLocation(program, b"u_proj\0".as_ptr() as _),
				view_loc: gl::GetUniformLocation(program, b"u_view\0".as_ptr() as _),
			})
		}
	}

	pub const fn invalid() -> Shader {
		Shader {
			gl_handle: 0,
			proj_loc: 0,
			view_loc: 0,
		}
	}

	fn get_currently_bound_raw() -> u32 {
		unsafe {
			let mut handle = 0;
			gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut handle);
			handle as u32
		}
	}

	pub fn use_program(&self) {
		unsafe {
			gl::UseProgram(self.gl_handle);
		}
	}

	pub fn is_bound(&self) -> bool {
		self.gl_handle == Shader::get_currently_bound_raw()
	}

	pub fn get_uniform_loc(&self, uniform: &str) -> i32 {
		use std::ffi::CString;

		unsafe {
			let cstr = CString::new(uniform).unwrap();
			gl::GetUniformLocation(self.gl_handle, cstr.as_ptr())
		}
	}

	pub fn set_uniform_vec2(&self, uniform: &str, v: Vec2) {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);

		unsafe {
			gl::Uniform2f(self.get_uniform_loc(&uniform), v.x, v.y);
		}
	}

	pub fn set_uniform_vec3<V>(&self, uniform: &str, v: V) where V: Into<Vec3> {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);

		unsafe {
			let v = v.into();
			gl::Uniform3f(self.get_uniform_loc(&uniform), v.x, v.y, v.z);
		}
	}

	pub fn set_uniform_vec4<V>(&self, uniform: &str, v: V) where V: Into<Vec4> {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);

		unsafe {
			let v = v.into();
			gl::Uniform4f(self.get_uniform_loc(&uniform), v.x, v.y, v.z, v.w);
		}
	}

	pub fn set_uniform_i32(&self, uniform: &str, v: i32) {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);

		unsafe {
			gl::Uniform1i(self.get_uniform_loc(&uniform), v);
		}		
	}

	pub fn set_uniform_f32(&self, uniform: &str, v: f32) {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);

		unsafe {
			gl::Uniform1f(self.get_uniform_loc(&uniform), v);
		}
	}
	
	pub fn set_uniform_mat_raw(&self, uniform: i32, mat: &Mat4) {
		assert!(self.is_bound(), "Tried to set uniform on unbound shader");

		unsafe {
			gl::UniformMatrix4fv(uniform, 1, 0, mat.transpose().rows.as_ptr() as *const f32);
		}
	}

	pub fn set_uniform_mat(&self, uniform: &str, mat: &Mat4) {
		assert!(self.is_bound(), "Tried to set uniform '{}' on unbound shader", uniform);
		self.set_uniform_mat_raw(self.get_uniform_loc(&uniform), &mat);
	}

	pub fn set_proj(&self, mat: &Mat4) {
		assert!(self.is_bound(), "Tried to set uniform 'u_proj' on unbound shader");
		self.set_uniform_mat_raw(self.proj_loc, &mat);
	}

	pub fn set_view(&self, mat: &Mat4) {
		assert!(self.is_bound(), "Tried to set uniform 'u_view' on unbound shader");
		self.set_uniform_mat_raw(self.view_loc, &mat);
	}
}