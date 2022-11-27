pub const TYPE_ALBUM: &str = "album";
pub const TYPE_TRACK: &str = "track";
pub const TYPE_ARTIST: &str = "artist";

pub(crate) mod object_type_serialize {
    use serde::{Deserialize, Deserializer, Serializer};

    use super::ObjectType;

    pub(crate) fn serialize<S, T>(_: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: ObjectType,
        S: Serializer,
    {
        serializer.serialize_str(T::OBJECT_TYPE)
    }

    pub(crate) fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        T: ObjectType + Default,
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        if s == T::OBJECT_TYPE {
            Ok(T::default())
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(&s),
                &T::OBJECT_TYPE,
            ))
        }
    }
}

pub(crate) trait ObjectType {
    const OBJECT_TYPE: &'static str;
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TypeAlbum;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TypeTrack;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct TypeArtist;

impl ObjectType for TypeAlbum {
    const OBJECT_TYPE: &'static str = TYPE_ALBUM;
}

impl ObjectType for TypeTrack {
    const OBJECT_TYPE: &'static str = TYPE_TRACK;
}

impl ObjectType for TypeArtist {
    const OBJECT_TYPE: &'static str = TYPE_ARTIST;
}
