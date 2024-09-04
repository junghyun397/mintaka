use crate::opening::opening_stage::OpeningStage;
use crate::opening::OpeningKind;

pub struct OpeningAgent {
    pub moves: u64,
    pub opening_kind: OpeningKind,
    pub opening_stage: OpeningStage,
}

impl OpeningAgent {

    fn next(mut self) -> Self {
        todo!()
    }

}
