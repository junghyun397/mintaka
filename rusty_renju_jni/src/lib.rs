use jni::sys::{jint, jlong};
use rusty_renju::notation::color::Color;

pub mod board_jni;
pub mod pattern_jni;

fn retrieve_ref<'a, T>(ptr: jlong) -> &'a mut T {
    unsafe { &mut *(ptr as *mut T) }
}

fn value_to_ptr<T>(value: T) -> jlong {
    Box::into_raw(Box::new(value)) as jlong
}

pub const COLOR_BLACK: jint = 1;
pub const COLOR_WHITE: jint = 2;
pub const COLOR_NONE: jint = 0;

fn jint_to_color(color: jint) -> Option<Color> {
    match color {
        COLOR_BLACK => Some(Color::Black),
        COLOR_WHITE => Some(Color::White),
        _ => None
    }
}

fn color_to_jint(color: Option<Color>) -> jint {
    match color {
        Some(Color::Black) => COLOR_BLACK,
        Some(Color::White) => COLOR_WHITE,
        _ => COLOR_NONE
    }
}
