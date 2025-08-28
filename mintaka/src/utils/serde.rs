use serde::Serializer;

pub fn serialize_array<S, T, const N: usize>(array: &[T; N], serializer: S) -> Result<S::Ok, S::Error> where
    T: serde::Serialize,
    S: Serializer
{
    serializer.collect_seq(array.iter())
}

pub fn deserialize_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error> where
    T: serde::Deserialize<'de> + std::fmt::Debug,
    D: serde::Deserializer<'de>
{
    let array: Vec<T> = serde::Deserialize::deserialize(deserializer)?;

    if array.len() != N {
        return Err(serde::de::Error::custom("array length is not equal to N"));
    }

    Ok(array.try_into().unwrap())
}
