use std::{borrow::Cow, fmt::Debug, marker::PhantomData};

use log::trace;
use reqwest::Method;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) use self::private::PageObject;
#[cfg(feature = "async")]
use crate::client::request_builder::AsyncRequestBuilder;
#[cfg(feature = "sync")]
use crate::client::request_builder::SyncRequestBuilder;
use crate::client::request_builder::{BaseRequestBuilderContainer, RequestBuilder, TryFromEmptyResponse};

mod private {
    use serde::{Deserialize, Serialize};

    /// A page object returned from Spotify.
    ///
    /// This object is only referenced through [Page] and the various wrapper types for paged information.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct PageObject<T>
    where
        T: Serialize,
    {
        pub items: Vec<T>,
        pub next: Option<String>,

        // these fields aren't actually needed but keep them around for logging purposes
        #[allow(dead_code)]
        limit: usize,
        #[allow(dead_code)]
        offset: usize,
        #[allow(dead_code)]
        total: usize,
    }
}

struct PageRequestBuilder<TClient, TInner>(RequestBuilder<TClient, TInner>);

/// A trait describing a page-like object that is returned from Spotify's search API.
///
/// You do not have to use this trait directly.
#[doc(hidden)]
pub trait PageInformation<T>
where
    Self: crate::private::Sealed,
{
    /// The iterator type this page contains.
    type Items: IntoIterator<Item = T>;

    /// Return the items in this page.
    fn items(&self) -> Self::Items;

    /// Return the items in this page while consuming the page.
    fn take_items(self) -> Self::Items;

    /// Returns the URL for the next page from this page, if it exists.
    fn next(self) -> Option<String>;
}

/// A page of items.
#[derive(Debug)]
pub struct Page<TInner, TItem>
where
    TInner: PageInformation<TItem> + DeserializeOwned + Debug,
{
    pub(crate) inner: TInner,
    pub(crate) phantom: PhantomData<TItem>,
}

impl<TClient, TInner> BaseRequestBuilderContainer<TClient, TInner> for PageRequestBuilder<TClient, TInner> {
    fn new<S>(method: Method, base_url: S, client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new(method, base_url, client))
    }

    fn new_with_body<S>(method: Method, base_url: S, body: (), client: TClient) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self(RequestBuilder::new_with_body(method, base_url, body, client))
    }

    fn take_base_builder(self) -> RequestBuilder<TClient, TInner> {
        self.0
    }

    fn get_base_builder_mut(&mut self) -> &mut RequestBuilder<TClient, TInner> {
        &mut self.0
    }
}

impl<T> crate::private::Sealed for PageObject<T> where T: Serialize {}

impl<TItem, TReturn> PageInformation<TReturn> for PageObject<TItem>
where
    TItem: ToOwned + TryInto<TReturn> + Serialize,
    TReturn: TryFrom<<TItem as ToOwned>::Owned>,
{
    type Items = Vec<TReturn>;

    fn items(&self) -> Self::Items {
        self.items
            .iter()
            .filter_map(|item| item.to_owned().try_into().ok())
            .collect()
    }

    fn take_items(self) -> Self::Items {
        self.items.into_iter().filter_map(|item| item.try_into().ok()).collect()
    }

    fn next(self) -> Option<String> {
        self.next
    }
}

impl<TInner, TItem> Page<TInner, TItem>
where
    TInner: PageInformation<TItem> + DeserializeOwned + Debug,
{
    /// Return the items in this page. The internal items will have to be cloned and converted into the return type.
    pub fn items(&self) -> TInner::Items {
        self.inner.items()
    }

    /// Return the items in this page while consuming the page. This helps avoid cloning the internal items, which may
    /// be quite large.
    pub fn take_items(self) -> TInner::Items {
        self.inner.take_items()
    }
}

#[cfg(feature = "async")]
impl<TInner, TItem> Page<TInner, TItem>
where
    TInner: PageInformation<TItem> + DeserializeOwned + Debug + TryFromEmptyResponse + Send + Sync,
{
    /// Return the next page from this page, if it exists.
    pub async fn next_page_async<C>(self, client: &'_ C) -> crate::error::Result<Option<Page<TInner, TItem>>>
    where
        C: crate::client::private::BuildHttpRequestAsync
            + crate::client::private::AccessTokenExpiryAsync
            + Clone
            + Send
            + Sync,
    {
        if let Some(url) = self.inner.next() {
            let next_page = PageRequestBuilder::new(Method::GET, url, client.clone())
                .send_async()
                .await?;
            trace!("Next page: {next_page:?}");

            Ok(Some(Page {
                inner: next_page,
                phantom: PhantomData,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(feature = "sync")]
impl<TInner, TItem> Page<TInner, TItem>
where
    TInner: PageInformation<TItem> + DeserializeOwned + Debug + TryFromEmptyResponse,
{
    /// Return the next page from this page, if it exists.
    pub fn next_page_sync<C>(self, client: &'_ C) -> crate::error::Result<Option<Page<TInner, TItem>>>
    where
        C: crate::client::private::BuildHttpRequestSync + crate::client::private::AccessTokenExpirySync + Clone,
    {
        if let Some(url) = self.inner.next() {
            let next_page = PageRequestBuilder::new(Method::GET, url, client.clone()).send_sync()?;
            trace!("Next page: {next_page:?}");

            Ok(Some(Page {
                inner: next_page,
                phantom: PhantomData,
            }))
        } else {
            Ok(None)
        }
    }
}
