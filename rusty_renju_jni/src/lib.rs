use jni::objects::JClass;
use jni::JNIEnv;

#[no_mangle]
pub extern "system" fn Java_RustyRenju_hello(
    _env: JNIEnv,
    _class: JClass,
) {
    todo!()
}
