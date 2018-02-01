#![allow(dead_code)]

use std;
use math::*;
use rendering::gl;

use std::fmt::Write;

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

			let mut status = 0i32;
			gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
			if status == 0 {
				let mut buf = [0u8; 1024];
				let mut len = 0;
				gl::GetProgramInfoLog(program, buf.len() as _, &mut len, buf.as_mut_ptr() as _);

				return Err(CStr::from_bytes_with_nul_unchecked(&buf[..len as usize]).to_string_lossy().into());
			}

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


pub struct ShaderBuilder {
	attributes: Vec<String>,
	varyings: Vec<String>,
	uniforms: Vec<String>,

	vertex_body: String,
	fragment_body: String,

	use_3d: bool,
	use_proj: bool,
	use_view: bool,
	use_highp: bool,
}

impl ShaderBuilder {
	pub fn new() -> Self {
		ShaderBuilder {
			attributes: Vec::new(),
			varyings: Vec::new(),
			uniforms: Vec::new(),

			vertex_body: String::new(),
			fragment_body: String::new(),

			use_3d: false,
			use_proj: false,
			use_view: false,
			use_highp: false,
		}
	}

	pub fn use_3d(mut self) -> Self { self.use_3d = true; self }
	pub fn use_proj(mut self) -> Self { self.use_proj = true; self.uniform("proj", "mat4") }
	pub fn use_view(mut self) -> Self { self.use_view = true; self.uniform("view", "mat4") }
	pub fn use_highp(mut self) -> Self { self.use_highp = true; self }

	pub fn vertex(mut self, data: &str) -> Self {
		write!(&mut self.vertex_body, "{};\n", data).unwrap(); self
	}

	pub fn fragment(mut self, data: &str) -> Self {
		write!(&mut self.fragment_body, "{};\n", data).unwrap(); self
	}

	pub fn uniform(mut self, name: &str, ty: &str) -> Self {
		self.uniforms.push(format!("{} u_{}", ty, name)); self
	}

	pub fn attribute(mut self, name: &str, ty: &str) -> Self {
		if name == "position" {
			println!("Tried to overwrite 'position' attribute while building shader - ignoring");
			return self
		}

		self.attributes.push(format!("{} {}", ty, name)); self
	}

	pub fn varying(mut self, name: &str, ty: &str) -> Self {
		self.varyings.push(format!("{} v_{}", ty, name)); self
	}

	pub fn frag_attribute(mut self, name: &str, ty: &str) -> Self {
		self.attributes.push(format!("{} {}", ty, name));
		self.varyings.push(format!("{} v_{}", ty, name));

		write!(&mut self.vertex_body, "v_{} = {};\n", name, name).unwrap();

		self
	}

	pub fn output(mut self, expr: &str) -> Self {
		write!(&mut self.fragment_body, "gl_FragColor = {};\n", expr).unwrap();
		self
	}

	pub fn finalize_source(mut self) -> (String, String) {
		let mut varyings_and_uniforms = String::new();

		for v in self.varyings.iter() { write!(&mut varyings_and_uniforms, "varying {};\n", v).unwrap(); }
		for u in self.uniforms.iter() { write!(&mut varyings_and_uniforms, "uniform {};\n", u).unwrap(); }

		let mut vert_src = String::new();
		let mut frag_src = String::new();

		let position_attr_ty = if self.use_3d { "vec3" } else { "vec2" };

		write!(&mut vert_src, "attribute {} position;\n", position_attr_ty).unwrap();
		for a in self.attributes.iter() { write!(&mut vert_src, "attribute {};\n", a).unwrap(); }

		let precision = if self.use_highp { "highp" } else { "mediump" };

		self.vertex_body.push_str("gl_Position = ");
		if self.use_proj { self.vertex_body.push_str("u_proj * "); }
		if self.use_view { self.vertex_body.push_str("u_view * "); }
		if self.use_3d {
			self.vertex_body.push_str("vec4(position, 1.0);\n");
		} else {
			self.vertex_body.push_str("vec4(position, 0.0, 1.0);\n");
		}

		let mut bodies = [&mut self.vertex_body, &mut self.fragment_body];
		for (sh, body) in [&mut vert_src, &mut frag_src].iter_mut().zip(bodies.iter_mut()) {
			write!(sh, "precision {} float;\n{}\n",
				precision, varyings_and_uniforms).unwrap();

			let mut position = 0;

			while let Some(start) = body[position..].find("func ") {
				let length = body[start..].chars()
					.scan((false, 0), |acc, c| {
						let (body, nesting) = *acc;

						*acc = match (body, nesting, c) {
							(false, _, '}') => return None,
							(true, 1, '}') => return None,

							(false, 0, '{') => (true, 1),
							(true, x, '{') => (true, x+1),
							(true, x, '}') => (true, x-1),
							_ => *acc,
						};

						Some(*acc)
					})
					.count();

				let start = start + position;
				let end = start + length + 1;

				write!(sh, "{}\n", &body[start+5..end]).unwrap();
				body.splice(start..end, "");
				position = start;
			}

			write!(sh, "void main() {{\n{}}}\n", body).unwrap();
		}

		(vert_src, frag_src)
	}

	pub fn finalize(self) -> Result<Shader, String> {
		let (v,f) = self.finalize_source();
		Shader::new(&v, &f)
	}
}

#[cfg(test)] mod tests {
	#[test]
	fn shader_builder() {
		let (vsh, fsh) = ::ShaderBuilder::new()
			.uniform("tex", "sampler2D")
			.attribute("some_random_attribute", "vec4")
			.frag_attribute("color", "vec3")
			.frag_attribute("uv", "vec2")
			.use_proj() .use_view()
			.fragment("
				func vec3 function_test(vec3 c) {
					return vec3(1.0) - c;
				}

				func vec3 function_test_2(float c) {
					if (c < 0.5) {
						return vec3(c);
					} else {
						return vec3(1.0 - c);
					}
				}

				vec3 color = function_test(v_color);
				color.g = texture2D(u_tex, v_uv).r")
			.output("vec4(color, 1.0)")
			.finalize_source();

		println!("vert source\n==========\n{}\n", vsh);
		println!("frag source\n==========\n{}", fsh);

		let (vsh, fsh) = ::ShaderBuilder::new()
			.use_3d()
			.output("vec4(1.0)")
			.finalize_source();

		println!("vert source\n==========\n{}\n", vsh);
		println!("frag source\n==========\n{}", fsh);
	}
}