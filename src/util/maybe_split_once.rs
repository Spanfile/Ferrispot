pub trait MaybeSplitOnce {
    fn maybe_split_once(&self, split: char) -> (&str, Option<&str>);
}

impl MaybeSplitOnce for str {
    fn maybe_split_once(&self, split: char) -> (&str, Option<&str>) {
        let mut split = self.splitn(2, split);

        // splitn(2) returns at most two items, always returning at least one (the entire input)
        let left = split.next().unwrap();
        let right = split.next();

        (left, right)
    }
}
