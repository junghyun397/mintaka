use crate::retrieve_ref;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use rusty_renju::board::Board;
use rusty_renju::pattern::PatternUnit;

fn count_pattern<F>(board_ptr: jlong, pos: jint, is_black: jboolean, count: F) -> jint where F: FnOnce(&PatternUnit) -> u32 {
    let pattern = &retrieve_ref::<Board>(board_ptr).patterns.field[pos as usize];
    let unit = match is_black {
        1 => pattern.black_unit,
        _ => pattern.white_unit
    };

    count(&unit) as jint
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_isForbidden(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
) -> jboolean {
    retrieve_ref::<Board>(board_ptr).patterns.field[pos as usize].is_forbidden() as jboolean
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_forbiddenKind(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
) -> jint {
    let pattern = &retrieve_ref::<Board>(board_ptr).patterns.field[pos as usize];
    pattern.forbidden_kind()
        .map(|kind| kind as jint)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countOpenThree(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_open_threes)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countOpenFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_open_fours)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countCloseThree(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_close_threes)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countClosedFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_closed_fours)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countTotalFour(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_total_fours)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_countFive(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jint,
    is_black: jboolean,
) -> jint {
    count_pattern(board_ptr, pos, is_black, PatternUnit::count_fives)
}
