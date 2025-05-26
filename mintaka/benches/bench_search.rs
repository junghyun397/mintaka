#![feature(test)]

extern crate test;

mod bench_search {

    #[bench]
    fn search_1m_nodes(b: &mut test::Bencher) {
        b.iter(|| {
            // TODO
        });
    }

}
