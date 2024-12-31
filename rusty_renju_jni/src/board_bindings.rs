use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyte, jlong};
use jni::JNIEnv;
use rusty_renju::board::Board;
use std::str::FromStr;

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_fromString(
    mut env: JNIEnv,
    _class: JClass,
    source: JString,
) -> jlong {
    let source: String = env.get_string(&source).unwrap().into();

    Board::from_str(&source)
        .map(|board| {
            Box::into_raw(Box::new(board)) as jlong
        })
        .unwrap_or(jlong::MIN)
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_fromMoves(
    mut env: JNIEnv,
    _class: JClass,
    moves: JByteArray,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_fromEachColorMoves(
    mut env: JNIEnv,
    _class: JClass,
    black_moves: JByteArray,
    white_moves: JByteArray,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_toString(
    mut env: JNIEnv,
    class: JClass,
) -> JString {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_set(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_setMut(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte,
) {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_unset(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_unsetMut(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte
) {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_pass(
    mut env: JNIEnv,
    _class: JClass,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_getPattern(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_getStoneColor(
    mut env: JNIEnv,
    _class: JClass,
    pos: jbyte,
) -> jlong {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Board_getPlayer(
    mut env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) -> jlong {
    todo!()
}
