use std::marker::ConstParamTy;

//noinspection RsUnresolvedPath
#[derive(ConstParamTy, Eq, PartialEq)]
pub enum ThreadType {
    Main, Worker,
}
