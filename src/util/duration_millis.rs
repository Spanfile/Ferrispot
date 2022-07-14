use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Duration;

pub(crate) fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    duration.as_millis().serialize(serializer)
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Duration::from_millis(Deserialize::deserialize(deserializer)?))
}
