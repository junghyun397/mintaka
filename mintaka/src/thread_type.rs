use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Copy, Clone, Eq, PartialEq)]
pub enum ThreadType {
    Main, Worker
}
