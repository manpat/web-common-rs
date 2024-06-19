use std::ops::Coroutine;
use std::pin::Pin;
use bindings::emscripten::*;

pub fn set_coro_as_main_loop<T>(coro: T) -> ! where T: Coroutine<Yield=(), Return=()> {
	unsafe {
		let coro: Pin<Box<dyn Coroutine<Yield=(), Return=()>>> = Box::pin(coro);
		emscripten_set_main_loop_arg(Some(resume_main_coro), Box::into_raw(Box::new(coro)) as _, 0, 1)
	}
}

extern "C" fn resume_main_coro(ctx: *mut CVoid) {
	use std::mem::transmute;
	use std::ops::CoroutineState::*;

	let coro: &mut Pin<Box<dyn Coroutine<Yield=(), Return=()>>> = unsafe{ transmute(ctx) };

	unsafe {
		match coro.as_mut().resume(()) {
			Yielded(()) => {}
			Complete(()) => {
				println!("Main coro has returned");

				Box::from_raw(ctx as _);

				emscripten_cancel_main_loop();
			}
		}
	}
}

#[macro_export]
macro_rules! parameter_lerp {
	( ($rc_obj:expr).$param:ident -> $to:expr, $duration:tt @ $delay:expr, $ease:ident ) => {{
		let rc_obj = $rc_obj.clone();

		let from = rc_obj.borrow().$param;
		let to = $to;

		let delay_frames = ($delay * 60.0) as u32; 
		let num_frames = ($duration * 60.0) as u32;

		use common::coro::Coro;

		Coro::from(move || {
			for _ in 0..delay_frames { yield }

			for i in 0..num_frames {
				let prog = i as f32 / num_frames as f32;
				rc_obj.borrow_mut().$param = prog.$ease(from, to);
				yield;
			}

			rc_obj.borrow_mut().$param = to;
		})
	}};

	( ($rc_obj:expr).$param:ident -> $to:expr, $duration:expr, $ease:ident ) => {{
		parameter_lerp!( ($rc_obj).$param -> $to, $duration @ 0.0, $ease )
	}};

	( $rc_obj:ident.$param:ident -> $to:expr, $duration:tt, $ease:ident ) => {{
		parameter_lerp!( ($rc_obj).$param -> $to, $duration @ 0.0, $ease )
	}};
	( $rc_obj:ident.$param:ident -> $to:expr, $duration:tt @ $delay:expr, $ease:ident ) => {{
		parameter_lerp!( ($rc_obj).$param -> $to, $duration @ $delay, $ease )
	}};
}
