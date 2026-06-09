#[cfg(feature = "serde")]
use crate::notation::color::ColorContainer;

#[cfg(feature = "serde")]
pub fn serialize_array<S, T, const N: usize>(array: &[T; N], serializer: S) -> Result<S::Ok, S::Error> where
    T: serde::Serialize,
    S: serde::Serializer
{
    serializer.collect_seq(array.iter())
}

#[cfg(feature = "serde")]
pub fn deserialize_array<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error> where
    T: serde::Deserialize<'de> + std::fmt::Debug,
    D: serde::Deserializer<'de>
{
    let array: Vec<T> = serde::Deserialize::deserialize(deserializer)?;

    if array.len() != N {
        return Err(serde::de::Error::custom(format!("array length is not equal to {N}")));
    }

    Ok(array.try_into().unwrap())
}

#[cfg(feature = "serde")]
pub fn serialize_color_container_array<S, T, const N: usize>(array: &ColorContainer<[T; N]>, serializer: S) -> Result<S::Ok, S::Error> where
    T: serde::Serialize,
    S: serde::Serializer
{
    serializer.collect_seq(array.0.iter().flatten())
}

#[cfg(feature = "serde")]
pub fn deserialize_color_container_array<'de, D, T, const N: usize>(deserializer: D) -> Result<ColorContainer<[T; N]>, D::Error> where
    T: serde::Deserialize<'de> + std::fmt::Debug,
    D: serde::Deserializer<'de>
{
    let mut array: Vec<T> = serde::Deserialize::deserialize(deserializer)?;

    if array.len() != N * 2 {
        return Err(serde::de::Error::custom(format!("array length is not equal to {}", N * 2)));
    }

    let white = array.split_off(N);
    let black = array;

    Ok(ColorContainer::new(
        black.try_into().unwrap(),
        white.try_into().unwrap()
    ))
}
