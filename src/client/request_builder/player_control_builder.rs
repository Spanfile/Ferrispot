use std::borrow::Cow;

use reqwest::Method;

use crate::client::{
    object,
    request_builder::{BaseRequestBuilderContainer, RequestBuilder},
};

/// A base builder type for the various player control request builders.
pub struct BasePlayerControlRequestBuilder<TClient, TBody>(RequestBuilder<TClient, (), TBody>);
/// A builder type for playing a context.
pub struct PlayContextRequestBuilder<TClient>(RequestBuilder<TClient, (), object::PlayContextBody>);

/// A type alias for a builder type for playing one or more playable items.
pub type PlayItemsRequestBuilder<TClient> = BasePlayerControlRequestBuilder<TClient, object::PlayItemsBody>;
/// A type alias for the various player control requests.
pub type PlayerControlRequestBuilder<TClient> = BasePlayerControlRequestBuilder<TClient, ()>;

impl<TClient, TBody> BaseRequestBuilderContainer<TClient, (), TBody>
    for BasePlayerControlRequestBuilder<TClient, TBody>
{
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: TBody, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, (), TBody> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, (), TBody> {
        &mut self.0
    }
}

impl<TClient> BaseRequestBuilderContainer<TClient, (), object::PlayContextBody> for PlayContextRequestBuilder<TClient> {
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: object::PlayContextBody, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, (), object::PlayContextBody> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, (), object::PlayContextBody> {
        &mut self.0
    }
}

impl<TClient, TReturn> BasePlayerControlRequestBuilder<TClient, TReturn> {
    /// Target playback on a certain Spotify device in the user's account.
    pub fn device_id<S>(self, device_id: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.append_query(object::DEVICE_ID_QUERY, device_id.into())
    }
}

impl<TClient> PlayContextRequestBuilder<TClient> {
    /// Start playing a certain track from the context, identified by its zero-based index in the context.
    pub fn offset<U>(self, offset: U) -> Self
    where
        U: Into<u32>,
    {
        self.replace_body(|body| object::PlayContextBody {
            offset: object::PlayContextOffset {
                position: Some(offset.into()),
                ..body.offset
            },
            ..body
        })
    }
}
