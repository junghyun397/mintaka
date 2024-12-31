use jni::objects::JClass;
use jni::sys::{jboolean, jint, jobject};
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_isForbidden(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_forbiddenKind(
    _env: JNIEnv,
    _class: JClass,
) -> jobject {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_Pattern_getUnit(
    _env: JNIEnv,
    _class: JClass,
) -> jobject {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countOpenThree(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countOpenFour(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countCloseFour(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countClosedFour(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countTotalFour(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}

#[no_mangle]
pub extern "system" fn Java_RustyRenju_PatternUnit_countFive(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    todo!()
}
