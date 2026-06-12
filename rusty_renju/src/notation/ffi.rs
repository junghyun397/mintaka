pub fn try_from_raw_slice<T: TryFrom<u32>>(slice: *const u32, len: usize) -> Option<Vec<T>> {
    if len == 0 {
        return Some(vec![]);
    }

    if slice.is_null() {
        return None;
    }

    let mut acc = Vec::with_capacity(len);

    for &idx in unsafe { std::slice::from_raw_parts(slice, len) }.iter() {
        if let Ok(pos) = T::try_from(idx) {
            acc.push(pos)
        }
    }
    
    Some(acc)
}
