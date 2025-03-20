use crate::retrieve_ref;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::pattern::Pattern;

fn count_pattern<F>(board_ptr: jlong, pos: jint, is_black: jboolean, count: F) -> jint where F: FnOnce(&Pattern) -> u32 {
    let pattern = &retrieve_ref::<Board>(board_ptr).patterns.field.access(Color::Black)[pos as usize];

    count(pattern) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_isForbidden(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
) -> jboolean {
    retrieve_ref::<Board>(board_ptr).patterns.field.black[pos as usize].is_forbidden() as jboolean
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_forbiddenKind(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
) -> jint {
    retrieve_ref::<Board>(board_ptr).patterns.field.black[pos as usize]
        .forbidden_kind()
        .map(|kind| kind as jint)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countOpenThree(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_open_threes)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countOpenFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_open_fours)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countCloseThree(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_close_threes)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countClosedFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_closed_fours)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countTotalFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_total_fours)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countFive(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, Pattern::count_fives)
}
