use crate::{client::private::SendHttpRequest, error::Result};

use log::debug;
use reqwest::{Method, Url};
use serde::{de::DeserializeOwned, Deserialize};
use std::{fmt::Debug, marker::PhantomData};

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
    fn next(&self) -> Option<&str>;
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

/// A page object returned from Spotify.
///
/// This object is only referenced through [Page] and the various wrapper types for paged information.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub(crate) struct PageObject<T> {
    items: Vec<T>,
    next: Option<String>,

    // these fields aren't actually needed but keep them around for logging purposes
    #[allow(dead_code)]
    limit: usize,
    #[allow(dead_code)]
    offset: usize,
    #[allow(dead_code)]
    total: usize,
}

impl<T> crate::private::Sealed for PageObject<T> {}

impl<TItem, TReturn> PageInformation<TReturn> for PageObject<TItem>
where
    TItem: ToOwned + Into<TReturn>,
    TReturn: From<<TItem as ToOwned>::Owned>,
{
    type Items = Vec<TReturn>;

    fn items(&self) -> Self::Items {
        self.items.iter().map(|item| item.to_owned().into()).collect()
    }

    fn take_items(self) -> Self::Items {
        self.items.into_iter().map(|item| item.into()).collect()
    }

    fn next(&self) -> Option<&str> {
        self.next.as_deref()
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

    /// Return the next page from this page, if it exists.
    pub async fn next_page<'a, C>(self, client: &'a C) -> Result<Option<Page<TInner, TItem>>>
    where
        C: SendHttpRequest<'a>,
    {
        if let Some(url) = self.inner.next() {
            // this will only fail if Spotify returns a malformed URL
            // TODO: maybe it's an error case?
            let url = Url::parse(url).expect("failed to parse next page URL: malformed URL in Spotify response");

            let response = client.send_http_request(Method::GET, url).send().await?;
            debug!("Next page response: {:?}", response);

            response.error_for_status_ref()?;

            let next_page: TInner = response.json().await?;
            debug!("Next page: {:?}", next_page);

            Ok(Some(Page {
                inner: next_page,
                phantom: PhantomData,
            }))
        } else {
            Ok(None)
        }
    }
}
