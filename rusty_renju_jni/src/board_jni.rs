use crate::{color_to_jint, jint_to_color, retrieve_ref, value_to_ptr};
use jni::objects::{JByteArray, JClass, JString, ReleaseMode};
use jni::sys::{jboolean, jbyte, jint, jlong};
use jni::JNIEnv;
use rusty_renju::board::Board;
use rusty_renju::notation::color::Color;
use rusty_renju::notation::pos::Pos;
use std::str::FromStr;

fn board_to_ptr(board: Board) -> jlong {
    Box::into_raw(Box::new(board)) as jlong
}

fn jbytearray_to_moves(env: &mut JNIEnv, source: JByteArray) -> Vec<Pos> { unsafe {
    env.get_array_elements(&source, ReleaseMode::NoCopyBack).unwrap().iter()
        .map(|&x| Pos::from_index(x as u8))
        .collect()
} }

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_drop(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
) {
    let _ = unsafe { Box::from_raw(board_ptr as *mut Board) };
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_fromString(
    mut env: JNIEnv,
    _class: JClass,
    source: JString,
) -> jlong {
    let source: String = env.get_string(&source).unwrap().into();

    Board::from_str(&source)
        .map(|board| {
            board_to_ptr(board)
        })
        .unwrap_or(jlong::MIN)
}

#[no_mangle]
pub unsafe extern "system" fn Java_com_do1phin_rustyrenju_Board_fromMoves(
    mut env: JNIEnv,
    _class: JClass,
    moves: JByteArray,
) -> jlong {
    let mut board = Board::default();
    board.batch_set_mut(jbytearray_to_moves(&mut env, moves).as_slice());

    board_to_ptr(board)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_fromEachColorMoves(
    mut env: JNIEnv,
    _class: JClass,
    black_moves: JByteArray,
    white_moves: JByteArray,
    player: jint,
) -> jlong {
    let mut board = Board::default();
    let black_moves = jbytearray_to_moves(&mut env, black_moves).into_boxed_slice();
    let white_moves = jbytearray_to_moves(&mut env, white_moves).into_boxed_slice();
    board.batch_set_each_color_mut(black_moves, white_moves, jint_to_color(player).unwrap());

    board_to_ptr(board)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_toString<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    board_ptr: jlong,
) -> JString<'a> {
    env.new_string(retrieve_ref::<Board>(board_ptr).to_string()).unwrap()
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_toDetailedString<'a>(
    mut env: JNIEnv<'a>,
    _class: JClass<'a>,
    board_ptr: jlong,
) -> JString<'a> {
    env.new_string(retrieve_ref::<Board>(board_ptr).build_detailed_string()).unwrap()
}


#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_getStones(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
) -> jint {
    retrieve_ref::<Board>(board_ptr).stones as jint
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_set(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) -> jlong {
    let board = retrieve_ref::<Board>(board_ptr)
        .clone()
        .set(Pos::from_index(pos as u8));

    value_to_ptr(board)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_unset(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) -> jlong {
    let board = retrieve_ref::<Board>(board_ptr)
        .clone()
        .unset(Pos::from_index(pos as u8));

    value_to_ptr(board)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_pass(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
) -> jlong {
    let board = retrieve_ref::<Board>(board_ptr)
        .clone()
        .pass();

    value_to_ptr(board)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_setMut(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) {
    retrieve_ref::<Board>(board_ptr)
        .set_mut(Pos::from_index(pos as u8));
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_unsetMut(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte
) {
    retrieve_ref::<Board>(board_ptr)
        .unset_mut(Pos::from_index(pos as u8));
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_passMut(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
) {
    retrieve_ref::<Board>(board_ptr)
        .pass_mut();
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_getPattern(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) -> jlong {
    let pattern = retrieve_ref::<Board>(board_ptr).patterns.field[pos as u8 as usize].clone();
    value_to_ptr(pattern)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_getStoneColor(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
    pos: jbyte,
) -> jint {
    let color = retrieve_ref::<Board>(board_ptr).stone_kind(Pos::from_index(pos as u8));
    color_to_jint(color)
}

#[no_mangle]
pub extern "system" fn Java_com_do1phin_rustyrenju_Board_getPlayerIsBlack(
    _env: JNIEnv,
    _class: JClass,
    board_ptr: jlong,
) -> jboolean {
    (retrieve_ref::<Board>(board_ptr).player_color == Color::Black) as jboolean
}
