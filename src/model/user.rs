//! Everything related to users.
//!
//! Contains the three different kinds of users; [PrivateUser], [CurrentUser] and [PublicUser].
//!
//! - [PrivateUser]: contains all available information about the current user authenticated to the API. Retrieved from
//!   the [`current_user_profile`-function](crate::client::ScopedClient::current_user_profile) *if* the application has
//!   been granted the [`user-read-private`-scope](crate::scope::Scope::UserReadPrivate).
//! - [CurrentUser]: contains all non-private information about the current user authenticated to the API. Retrieved
//!   from the [`current_user_profile`-function](crate::client::ScopedClient::current_user_profile) *if* the application
//!   has been granted the [`user-read-email`-scope](crate::scope::Scope::UserReadEmail), but *not* the
//!   [`user-read-private`-scope](crate::scope::Scope::UserReadPrivate).
//! - [PublicUser]: contains all public information about a user. Retrieved from the
//!   [`user_profile-`function](crate::client::UnscopedClient::user_profile).
//!
//! Additionally, there is the [User] enum that encompasses all three kinds of users.
//!
//! The user object Spotify returns from the API is not directly available. The three user objects, or the [User] enum,
//! may be serialized to get almost all of the original API response back. The model strips certain unnecessary or
//! redundant fields from the response.

mod private {
    use serde::{Deserialize, Serialize};

    use super::{ExplicitContent, Followers};
    use crate::model::{
        id::{Id, UserId},
        object_type::{object_type_serialize, TypeUser},
        CountryCode, ExternalUrls, Image,
    };

    pub(super) trait CommonFields {
        fn common_fields(&self) -> &CommonUserFields;
    }

    pub(super) trait CurrentFields {
        fn current_fields(&self) -> &CurrentUserFields;
    }

    pub(super) trait PrivateFields {
        fn private_fields(&self) -> &PrivateUserFields;
    }

    /// This struct covers all the possible user responses from Spotify's API. It has a function that converts it into
    /// a [User], depending on which fields are set.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct UserObject {
        /// Fields available in every user.
        #[serde(flatten)]
        pub(crate) common: CommonUserFields,

        /// Fields only in the current user.
        #[serde(flatten)]
        pub(crate) current: Option<CurrentUserFields>,

        /// Fields only in the current private user.
        #[serde(flatten)]
        pub(crate) private: Option<PrivateUserFields>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct CommonUserFields {
        pub(crate) display_name: Option<String>,
        #[serde(default)]
        pub(crate) external_urls: ExternalUrls,
        pub(crate) followers: Followers,
        pub(crate) id: Id<'static, UserId>,
        #[serde(default)]
        pub(crate) images: Vec<Image>,

        #[serde(rename = "type", with = "object_type_serialize")]
        pub(crate) item_type: TypeUser,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct CurrentUserFields {
        pub(crate) email: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub(crate) struct PrivateUserFields {
        pub(crate) country: CountryCode,
        pub(crate) explicit_content: ExplicitContent,
        // TODO: this should really be an enum, but I don't know what all the variants could be. there's at least
        // "premium", "free" aka "open", but there's also something like "family" maybe? "duo", "student"? even
        // more?
        pub(crate) product: String,
    }
}

use serde::{Deserialize, Serialize};

use self::private::{CommonUserFields, CurrentUserFields, PrivateUserFields, UserObject};
use super::{
    id::{Id, UserId},
    CountryCode, ExternalUrls, Image,
};
use crate::{error::ConversionError, prelude::IdTrait};

/// Information about a user's followers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Followers {
    // the API documents a href parameter but says it's always null, so it's not included here
    pub total: u32,
}

/// Information about a user's explicit content settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplicitContent {
    /// When `true`, indicates that explicit content should not be played.
    pub filter_enabled: bool,
    /// When `true`, indicates that the explicit content setting is locked and can't be changed by the user.
    pub filter_locked: bool,
}

/// Functions for retrieving information that is common to every user type.
pub trait CommonUserInformation: crate::private::Sealed {
    /// The user's display name if available.
    fn display_name(&self) -> Option<&str>;
    /// The external URLs for the user.
    fn external_urls(&self) -> &ExternalUrls;
    /// Information about the user's followers.
    fn followers(&self) -> Followers;
    /// The user's ID.
    fn id(&self) -> Id<'_, UserId>;
    /// The user's images.
    fn images(&self) -> &[Image];
}

/// Functions for retrieving information only in the current user.
pub trait CurrentUserInformation: crate::private::Sealed {
    /// The user's email.
    fn email(&self) -> &str;
}

/// Functions for retrieving information only in the current private user.
pub trait PrivateUserInformation: crate::private::Sealed {
    /// The user's country.
    fn country(&self) -> CountryCode;
    /// The user's explicit content settings.
    fn explicit_content(&self) -> ExplicitContent;
    /// The user's subscription level.
    fn product(&self) -> &str;
}

impl<T> CommonUserInformation for T
where
    T: private::CommonFields + crate::private::Sealed,
{
    fn display_name(&self) -> Option<&str> {
        self.common_fields().display_name.as_deref()
    }

    fn external_urls(&self) -> &ExternalUrls {
        &self.common_fields().external_urls
    }

    fn followers(&self) -> Followers {
        self.common_fields().followers
    }

    fn id(&self) -> Id<'_, UserId> {
        self.common_fields().id.as_borrowed()
    }

    fn images(&self) -> &[Image] {
        &self.common_fields().images
    }
}

impl<T> CurrentUserInformation for T
where
    T: private::CurrentFields + crate::private::Sealed,
{
    fn email(&self) -> &str {
        &self.current_fields().email
    }
}

impl<T> PrivateUserInformation for T
where
    T: private::PrivateFields + crate::private::Sealed,
{
    fn country(&self) -> CountryCode {
        self.private_fields().country
    }

    fn explicit_content(&self) -> ExplicitContent {
        self.private_fields().explicit_content
    }

    fn product(&self) -> &str {
        &self.private_fields().product
    }
}

/// An enum that encompasses all user types.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum User {
    Private(PrivateUser),
    Current(CurrentUser),
    Public(PublicUser),
}

/// This struct's only purpose is to make serializing more efficient by holding only references to its data. When
/// attempting to serialize a user object, its fields will be passed as references to this object which is then
/// serialized. This avoids having to clone the entire user in order to reconstruct a UserObject.
#[derive(Serialize)]
struct UserObjectRef<'a> {
    #[serde(flatten)]
    common: &'a CommonUserFields,
    #[serde(flatten)]
    current: Option<&'a CurrentUserFields>,
    #[serde(flatten)]
    private: Option<&'a PrivateUserFields>,
}

/// Private information about the current user. Contains [private information](self::PrivateUserInformation), in
/// addition to all [common](self::CommonUserInformation) and [current](self::CurrentUserInformation) user information
/// about the user.
#[derive(Debug, Clone, Eq, Deserialize)]
#[serde(try_from = "UserObject")]
pub struct PrivateUser {
    common: CommonUserFields,
    current: CurrentUserFields,
    private: PrivateUserFields,
}

/// Public information about the current user. Contains all [common](self::CommonUserInformation) and
/// [current](self::CurrentUserInformation) user information about the user.
#[derive(Debug, Clone, Eq, Deserialize)]
#[serde(try_from = "UserObject")]
pub struct CurrentUser {
    common: CommonUserFields,
    current: CurrentUserFields,
}

/// Public information about a user. Contains only the information [common to every user](self::CommonUserInformation).
#[derive(Debug, Clone, Eq, Deserialize)]
#[serde(try_from = "UserObject")]
pub struct PublicUser {
    common: CommonUserFields,
}

impl PartialEq for PrivateUser {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for CurrentUser {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for PublicUser {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<CurrentUser> for PrivateUser {
    fn eq(&self, other: &CurrentUser) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<PublicUser> for PrivateUser {
    fn eq(&self, other: &PublicUser) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<PrivateUser> for CurrentUser {
    fn eq(&self, other: &PrivateUser) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<PublicUser> for CurrentUser {
    fn eq(&self, other: &PublicUser) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<PrivateUser> for PublicUser {
    fn eq(&self, other: &PrivateUser) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq<CurrentUser> for PublicUser {
    fn eq(&self, other: &CurrentUser) -> bool {
        self.id() == other.id()
    }
}

impl TryFrom<UserObject> for User {
    type Error = ConversionError;

    fn try_from(obj: UserObject) -> Result<Self, Self::Error> {
        match (obj.current, obj.private) {
            (Some(current), Some(private)) => Ok(Self::Private(PrivateUser {
                common: obj.common,
                current,
                private,
            })),

            (Some(current), None) => Ok(Self::Current(CurrentUser {
                common: obj.common,
                current,
            })),

            (None, None) => Ok(Self::Public(PublicUser { common: obj.common })),

            (current, private) => Err(ConversionError(
                format!(
                    "impossible case trying to convert UserObject into User: current user fields is {current:?} while \
                     private user fields is {private:?}"
                )
                .into(),
            )),
        }
    }
}

impl From<PrivateUser> for User {
    fn from(private: PrivateUser) -> Self {
        Self::Private(private)
    }
}

impl From<CurrentUser> for User {
    fn from(current: CurrentUser) -> Self {
        Self::Current(current)
    }
}

impl From<PublicUser> for User {
    fn from(public: PublicUser) -> Self {
        Self::Public(public)
    }
}

impl TryFrom<User> for PrivateUser {
    type Error = ConversionError;

    fn try_from(user: User) -> Result<Self, Self::Error> {
        match user {
            User::Private(private) => Ok(private),

            User::Current(_) => Err(ConversionError(
                "attempt to convert current user into private user".into(),
            )),

            User::Public(_) => Err(ConversionError(
                "attempt to convert public user into private user".into(),
            )),
        }
    }
}

impl TryFrom<UserObject> for PrivateUser {
    type Error = ConversionError;

    fn try_from(obj: UserObject) -> Result<Self, Self::Error> {
        match (obj.current, obj.private) {
            (Some(current), Some(private)) => Ok(PrivateUser {
                common: obj.common,
                current,
                private,
            }),

            (current, private) => Err(ConversionError(
                format!(
                    "attempt to convert non-private user object into private user (current user fields is \
                     {current:?}, private user fields is {private:?})"
                )
                .into(),
            )),
        }
    }
}

impl TryFrom<User> for CurrentUser {
    type Error = ConversionError;

    fn try_from(user: User) -> Result<Self, Self::Error> {
        match user {
            User::Private(private) => Ok(CurrentUser {
                common: private.common,
                current: private.current,
            }),

            User::Current(current) => Ok(current),

            User::Public(_) => Err(ConversionError(
                "attempt to convert public user into current user".into(),
            )),
        }
    }
}

impl TryFrom<UserObject> for CurrentUser {
    type Error = ConversionError;

    fn try_from(obj: UserObject) -> Result<Self, Self::Error> {
        if let Some(current) = obj.current {
            Ok(CurrentUser {
                common: obj.common,
                current,
            })
        } else {
            Err(ConversionError(
                "attempt to convert public user object into current user".into(),
            ))
        }
    }
}

impl From<User> for PublicUser {
    fn from(user: User) -> Self {
        match user {
            User::Private(private) => PublicUser { common: private.common },
            User::Current(current) => PublicUser { common: current.common },
            User::Public(public) => public,
        }
    }
}

impl From<UserObject> for PublicUser {
    fn from(obj: UserObject) -> Self {
        PublicUser { common: obj.common }
    }
}

impl From<PrivateUser> for UserObject {
    fn from(value: PrivateUser) -> Self {
        Self {
            common: value.common,
            current: Some(value.current),
            private: Some(value.private),
        }
    }
}

impl From<CurrentUser> for UserObject {
    fn from(value: CurrentUser) -> Self {
        Self {
            common: value.common,
            current: Some(value.current),
            private: None,
        }
    }
}

impl From<PublicUser> for UserObject {
    fn from(value: PublicUser) -> Self {
        Self {
            common: value.common,
            current: None,
            private: None,
        }
    }
}

impl crate::private::Sealed for PrivateUser {}
impl crate::private::Sealed for CurrentUser {}
impl crate::private::Sealed for PublicUser {}

impl private::CommonFields for PrivateUser {
    fn common_fields(&self) -> &CommonUserFields {
        &self.common
    }
}

impl private::CommonFields for CurrentUser {
    fn common_fields(&self) -> &CommonUserFields {
        &self.common
    }
}

impl private::CommonFields for PublicUser {
    fn common_fields(&self) -> &CommonUserFields {
        &self.common
    }
}

impl private::CurrentFields for PrivateUser {
    fn current_fields(&self) -> &CurrentUserFields {
        &self.current
    }
}

impl private::CurrentFields for CurrentUser {
    fn current_fields(&self) -> &CurrentUserFields {
        &self.current
    }
}

impl private::PrivateFields for PrivateUser {
    fn private_fields(&self) -> &PrivateUserFields {
        &self.private
    }
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            User::Private(private) => private.serialize(serializer),
            User::Current(current) => current.serialize(serializer),
            User::Public(public) => public.serialize(serializer),
        }
    }
}

impl Serialize for PrivateUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        UserObjectRef {
            common: &self.common,
            current: Some(&self.current),
            private: Some(&self.private),
        }
        .serialize(serializer)
    }
}

impl Serialize for CurrentUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        UserObjectRef {
            common: &self.common,
            current: Some(&self.current),
            private: None,
        }
        .serialize(serializer)
    }
}

impl Serialize for PublicUser {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        UserObjectRef {
            common: &self.common,
            current: None,
            private: None,
        }
        .serialize(serializer)
    }
}

// TODO: unit tests for all the various functions here. deserializing, serializing, equality between users, conversion
// between users
