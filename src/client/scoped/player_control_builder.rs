use std::borrow::Cow;

use reqwest::Method;

use super::private::{PlayContextBody, PlayContextOffset};
use crate::client::{
    request_builder::RequestBuilder,
    scoped::{BaseRequestBuilderContainer, DEVICE_ID_QUERY},
};

pub struct BasePlayerControlRequestBuilder<TClient, TBody>(RequestBuilder<TClient, (), TBody>);
pub struct PlayContextRequestBuilder<TClient>(RequestBuilder<TClient, (), super::private::PlayContextBody>);

pub type PlayItemsRequestBuilder<TClient> = BasePlayerControlRequestBuilder<TClient, super::private::PlayItemsBody>;
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

impl<TClient> BaseRequestBuilderContainer<TClient, (), super::private::PlayContextBody>
    for PlayContextRequestBuilder<TClient>
{
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: super::private::PlayContextBody, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, (), super::private::PlayContextBody> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, (), super::private::PlayContextBody> {
        &mut self.0
    }
}

impl<TClient, TReturn> BasePlayerControlRequestBuilder<TClient, TReturn> {
    /// Target playback on a certain Spotify device in the user's account.
    pub fn device_id<S>(self, device_id: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        self.append_query(DEVICE_ID_QUERY, device_id.into())
    }
}

impl<TClient> PlayContextRequestBuilder<TClient> {
    pub fn offset<U>(self, offset: U) -> Self
    where
        U: Into<u32>,
    {
        self.replace_body(|body| PlayContextBody {
            offset: PlayContextOffset {
                position: Some(offset.into()),
                ..body.offset
            },
            ..body
        })
    }
}
