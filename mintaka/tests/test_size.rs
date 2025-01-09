#[cfg(test)]
mod test_size {
    use mintaka::memo::tt_entry::TTEntryBucket;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<TTEntryBucket>(), 8 * 3)
    }

}
