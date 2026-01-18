#[cfg(test)]
mod test_image {
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos::pos_unchecked;
    use rusty_renju_image::{rusty_renju_image_format_png, rusty_renju_image_render, rusty_renju_image_renderer_sequence, HistoryRender, RenderPayloads};

    #[test]
    fn default_image() {
        let history = History::default()
            .set(pos_unchecked("h8"))
            .set(pos_unchecked("g7"))
            ;

        let board = Board::from(&history);
        let opts = RenderPayloads {
            history,
            history_render: HistoryRender::Sequence,
            offers: &[
                pos_unchecked("a1"),
                pos_unchecked("a2"),
            ],
            blinds: &[
                pos_unchecked("a9"),
            ],
            enable_forbidden: true,
        };

        let png = rusty_renju_image_render(
            rusty_renju_image_format_png(), 1.0,
            rusty_renju_image_renderer_sequence(), true,
            Box::into_raw(Box::new(board)),
            std::ptr::null(), 0,
            std::ptr::null(), 0,
            std::ptr::null(), 0,
        );

        let png: Vec<u8> = png.into();

        std::fs::write("test_output.png", png).unwrap();
    }

}
