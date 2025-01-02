use crate::retrieve_ref;
use jni::objects::JClass;
use jni::sys::{jboolean, jint, jlong};
use jni::JNIEnv;
use rusty_renju::notation::color::Color;
use rusty_renju::pattern::{Pattern, PatternUnit};

#[no_mangle]
pub unsafe extern "system" fn Java_RustyRenju_Pattern_destroy(
    _env: JNIEnv,
    _class: JClass,
    pattern_ptr: jlong,
) {
    let _ = Box::from_raw(pattern_ptr as *mut Pattern);
}

#[no_mangle]
pub unsafe extern "system" fn Java_RustyRenju_PatternUnit_destroy(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) {
    let _ = Box::from_raw(pattern_unit_ptr as *mut PatternUnit);
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_isForbidden(
    _env: JNIEnv,
    _class: JClass,
    pattern_ptr: jlong,
) -> jboolean {
    retrieve_ref::<Pattern>(pattern_ptr).is_forbidden() as jboolean
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_forbiddenKind(
    _env: JNIEnv,
    _class: JClass,
    pattern_ptr: jlong,
) -> jint {
    let pattern = retrieve_ref::<Pattern>(pattern_ptr);
    pattern.forbidden_kind()
        .map(|kind| kind as jint)
        .unwrap_or(0)
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_getUnit(
    _env: JNIEnv,
    _class: JClass,
    pattern_ptr: jlong,
) -> jlong {
    let pattern = retrieve_ref::<Pattern>(pattern_ptr);
    Box::into_raw(Box::new(pattern.access_unit(Color::Black))) as jlong
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countOpenThree(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_open_threes() as jint
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countOpenFour(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_open_fours() as jint
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countCloseFour(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_closed_fours() as jint
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countClosedFour(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_closed_fours() as jint
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countTotalFour(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_total_fours() as jint
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countFive(
    _env: JNIEnv,
    _class: JClass,
    pattern_unit_ptr: jlong,
) -> jint {
    retrieve_ref::<PatternUnit>(pattern_unit_ptr).count_fives() as jint
}
