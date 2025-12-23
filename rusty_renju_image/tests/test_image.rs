#[cfg(test)]
mod test_image {
    use rusty_renju::board::Board;
    use rusty_renju::history::History;
    use rusty_renju::notation::pos::pos_unchecked;
    use rusty_renju_image::{render_png, HistoryRender, RenderPayloads};

    #[test]
    fn default_image() {
        let history = History::default()
            .set(pos_unchecked("h8"))
            .set(pos_unchecked("g7"))
            ;

        let board = Board::from(&history);
        let opts = RenderPayloads {
            history: &history,
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

        let png = render_png(&board, opts);

        std::fs::write("test_output.png", &png).unwrap();
    }

}
