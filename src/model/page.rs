use serde::Deserialize;

// pub struct Pages<'a, T>
// where
//     T: ApiObject,
//     <T as ApiObject>::SourceObject: Into<T>,
// {
//     // current_page: Page<<T as ApiObject>::SourceObject>,
//     current_page: IteratorPage<<T as ApiObject>::SourceObject>,
//     client: &'a dyn SendHttpRequest,
// }

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Page<O> {
    pub items: Vec<O>,
    pub limit: usize,
    pub offset: usize,
    pub total: usize,
}

impl<O> IntoIterator for Page<O> {
    type Item = O;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

// impl<'a, T> Pages<'a, T>
// where
//     T: ApiObject,
//     <T as ApiObject>::SourceObject: 'static + Into<T>,
// {
//     pub(crate) fn new(first_page: Page<<T as ApiObject>::SourceObject>, client: &'a dyn SendHttpRequest) -> Self {
//         Self {
//             current_page: IteratorPage {
//                 len: first_page.items.len(),
//                 items: Box::new(first_page.items.into_iter()),
//                 // limit: first_page.limit,
//                 // offset: first_page.offset,
//                 total: first_page.total,
//                 next: first_page.next,
//             },
//             client,
//         }
//     }
// }
