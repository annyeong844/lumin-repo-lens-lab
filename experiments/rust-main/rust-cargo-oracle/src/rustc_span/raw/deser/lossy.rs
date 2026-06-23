use serde::Deserialize;

pub(super) struct Lossy<T>(pub(super) Option<T>);

impl<'de, T> Deserialize<'de> for Lossy<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(T::deserialize(deserializer).ok()))
    }
}
