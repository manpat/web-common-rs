use std::mem::transmute;

use bindings::emscripten::*;
use common::math::vector::*;

use std::ffi::CStr;


#[derive(Copy, Clone, Debug)]
pub enum KeyCode {
	Space, Tab, Backspace,
	Insert, Delete,
	PageUp, PageDown,
	Home, End,
	Escape, Enter,

	Shift, Control,

	Left, Right, Up, Down,

	Digit(i32),
	Symbol(char),
	Alpha(char),
	F(i32),
}


pub enum Event {
	Resize(Vec2i),

	Down(Vec2i),
	Up(Vec2i),
	Move(Vec2i),

	KeyDown(KeyCode),
	KeyUp(KeyCode),
}


pub unsafe fn initialise_ems_event_queue(queue: &mut Vec<Event>) {
	use std::ptr::null;

	js! { b"Module.canvas = document.getElementById('canvas')\0" };

	let evt_ptr = transmute(queue);

	let window_target = b"#window\0".as_ptr() as _;
	let canvas_target = b"#canvas\0".as_ptr() as _;

	on_resize(0, null(), evt_ptr);
	emscripten_set_resize_callback(window_target, evt_ptr, 1, Some(on_resize));

	emscripten_set_mousemove_callback(canvas_target, evt_ptr, 1, Some(on_mouse_move));
	emscripten_set_mousedown_callback(canvas_target, evt_ptr, 1, Some(on_mouse_down));
	emscripten_set_mouseup_callback(canvas_target, evt_ptr, 1, Some(on_mouse_up));

	emscripten_set_touchstart_callback(canvas_target, evt_ptr, 1, Some(on_touch_start));
	emscripten_set_touchmove_callback(canvas_target, evt_ptr, 1, Some(on_touch_move));
	emscripten_set_touchend_callback(canvas_target, evt_ptr, 1, Some(on_touch_end));
	emscripten_set_touchcancel_callback(canvas_target, evt_ptr, 1, Some(on_touch_end));

	emscripten_set_keydown_callback(window_target, evt_ptr, 1, Some(on_key_down));
	emscripten_set_keyup_callback(window_target, evt_ptr, 1, Some(on_key_up));
}

unsafe extern "C"
fn on_resize(_: i32, _e: *const EmscriptenUiEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);

	let canvas_target = b"#canvas\0".as_ptr() as _;
	let (mut width, mut height) = (0.0, 0.0);

	emscripten_get_element_css_size(canvas_target, &mut width, &mut height);
	emscripten_set_canvas_size(width as i32, height as i32);

	event_queue.push(Event::Resize(Vec2i::new(width as i32, height as i32)));
	
	1
}


unsafe extern "C"
fn on_mouse_move(_: i32, e: *const EmscriptenMouseEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenMouseEvent = transmute(e);

	event_queue.push(Event::Move(Vec2i::new(e.clientX as _, e.clientY as _)));
	
	1
}
unsafe extern "C"
fn on_mouse_down(_: i32, e: *const EmscriptenMouseEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenMouseEvent = transmute(e);

	event_queue.push(Event::Down(Vec2i::new(e.clientX as _, e.clientY as _)));
	
	1
}
unsafe extern "C"
fn on_mouse_up(_: i32, e: *const EmscriptenMouseEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenMouseEvent = transmute(e);

	event_queue.push(Event::Up(Vec2i::new(e.clientX as _, e.clientY as _)));
	
	1
}


unsafe extern "C"
fn on_touch_move(_: i32, e: *const EmscriptenTouchEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenTouchEvent = transmute(e);

	if e.touches[0].identifier != 0 { return 0 }

	let pos = Vec2i::new(e.touches[0].clientX as _, e.touches[0].clientY as _);
	event_queue.push(Event::Move(pos));
	
	1
}

unsafe extern "C"
fn on_touch_start(_: i32, e: *const EmscriptenTouchEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenTouchEvent = transmute(e);

	if e.touches[0].identifier != 0 { return 0 }

	let pos = Vec2i::new(e.touches[0].clientX as _, e.touches[0].clientY as _);
	event_queue.push(Event::Down(pos));
	
	1
}

unsafe extern "C"
fn on_touch_end(_: i32, e: *const EmscriptenTouchEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenTouchEvent = transmute(e);

	if e.numTouches > 1 {
		use std::mem::uninitialized;

		// TODO: Make this requestable
		let mut fse: EmscriptenFullscreenChangeEvent = uninitialized();
		emscripten_get_fullscreen_status(&mut fse);

		if fse.isFullscreen == 0 {
			js!{ b"Module.requestFullscreen(1,1,0)" };
		}
	}

	if e.touches[0].identifier != 0 { return 0 }

	let pos = Vec2i::new(e.touches[0].clientX as _, e.touches[0].clientY as _);
	event_queue.push(Event::Up(pos));
	
	1
}


unsafe extern "C"
fn on_key_down(_: i32, e: *const EmscriptenKeyboardEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenKeyboardEvent = transmute(e);

	if e.repeat != 0 { return 1 }

	let keycode = KeyCode::from_c_str(e.code.as_ptr(), e.key.as_ptr());

	match keycode {
		Some(KeyCode::F(5))
		| Some(KeyCode::F(6))
		| Some(KeyCode::F(12)) => {
			return 0
		}

		_ => {}
	}

	// println!("{:?} - {:?}, {:?}", keycode, CStr::from_ptr(e.key.as_ptr()),
	// 	CStr::from_ptr(e.code.as_ptr()));

	if let Some(keycode) = keycode {
		event_queue.push(Event::KeyDown(keycode));
		1
	} else {
		0
	}
}

unsafe extern "C"
fn on_key_up(_: i32, e: *const EmscriptenKeyboardEvent, ud: *mut CVoid) -> i32 {
	let event_queue: &mut Vec<Event> = transmute(ud);
	let e: &EmscriptenKeyboardEvent = transmute(e);

	let keycode = KeyCode::from_c_str(e.code.as_ptr(), e.key.as_ptr());

	if let Some(keycode) = keycode {
		event_queue.push(Event::KeyUp(keycode));
		1
	} else {
		0
	}
}


impl KeyCode {
	pub fn from_c_str(code_str: *const CChar, key_str: *const CChar) -> Option<KeyCode> {
		use std::ascii::AsciiExt;

		let code = unsafe{ CStr::from_ptr(code_str) };
		let key = unsafe{ CStr::from_ptr(key_str) };

		let code = code.to_str().unwrap();

		match code {
			"Space" => return Some(KeyCode::Space),

			"Insert" => return Some(KeyCode::Insert),
			"Delete" => return Some(KeyCode::Delete),
			"PageUp" => return Some(KeyCode::PageUp),
			"PageDown" => return Some(KeyCode::PageDown),

			"Home" => return Some(KeyCode::Home),
			"End" => return Some(KeyCode::End),
			"Escape" => return Some(KeyCode::Escape),
			"Enter" => return Some(KeyCode::Enter),

			"Tab" => return Some(KeyCode::Tab),
			"Backspace" => return Some(KeyCode::Backspace),

			"ArrowLeft" => return Some(KeyCode::Left),
			"ArrowRight" => return Some(KeyCode::Right),
			"ArrowUp" => return Some(KeyCode::Up),
			"ArrowDown" => return Some(KeyCode::Down),

			x if x.len() == 4 && x.starts_with("Key") => {
				return Some(KeyCode::Alpha(x.chars().nth(3).unwrap()))
			}

			x if x.starts_with("Digit") => {
				return code[5..].parse().ok().map(|n| KeyCode::Digit(n))
			}

			_ => {}
		}

		let key = key.to_str().unwrap();

		match key {
			"Shift" => return Some(KeyCode::Shift),
			"Control" => return Some(KeyCode::Control),

			"F1" | "F2" | "F3" | "F4" | "F5" | "F6" | "F7" | "F8"
			| "F9" | "F10" | "F11" | "F12" => {
				return Some(KeyCode::F( key[1..].parse().unwrap() ))
			}

			x if x.is_ascii_punctuation() => {
				return Some(KeyCode::Symbol(x.chars().next().unwrap()))
			}

			_ => {}
		}

		None
	}
}