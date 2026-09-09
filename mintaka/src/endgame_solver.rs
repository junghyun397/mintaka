use crate::config::{Config, SearchObjective};
use crate::eval::evaluator::{ActiveEvaluator, Evaluator};
use crate::game_state::GameState;
use crate::memo::history_table::HistoryTable;
use crate::memo::transposition_table::TranspositionTable;
use crate::search_endgame;
use crate::search_endgame::ThreatSearchKind;
use crate::thread_data::ThreadData;
use crate::thread_type::WorkerThread;
use rusty_renju::notation::pos::Pos;
use rusty_renju::notation::rule::RuleKind;
use rusty_renju::utils::byte_size::ByteSize;
use rusty_renju::utils::empty::Empty;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32};
use std::time::Instant;
use crate::utils::depth::Depth;

pub struct EndgameSolution {
    pub sequence: Option<Vec<Pos>>,
    pub nodes: u32,
}

pub fn solve_endgame<const R: RuleKind>(
    mut state: GameState<R>,
    threat_kind: ThreatSearchKind,
    depth_limit: Option<Depth>,
    global_counter_1k: Arc<AtomicU32>,
    aborted: Arc<AtomicBool>,
) -> EndgameSolution {
    let mut config = Config::default();
    
    config.max_quiescence_depth = depth_limit;
    
    let tt = TranspositionTable::new_with_size(ByteSize::from_kib(32));
    let ht = HistoryTable::empty();

    let evaluator = ActiveEvaluator::from_state(&state);

    let mut td = ThreadData::new(
        WorkerThread::<Instant>::new(), 0,
        SearchObjective::Best, config,
        evaluator,
        tt.view(),
        ht,
        &aborted,
        &global_counter_1k,
    );
    
    let sequence = match threat_kind {
        ThreatSearchKind::VCF => search_endgame::endgame_proof::<R, { ThreatSearchKind::VCF }>(&mut td, &mut state),
        ThreatSearchKind::VCT => search_endgame::endgame_proof::<R, { ThreatSearchKind::VCT }>(&mut td, &mut state),
        ThreatSearchKind::Forced => search_endgame::endgame_proof::<R, { ThreatSearchKind::Forced }>(&mut td, &mut state),
    };

    EndgameSolution { sequence, nodes: td.batch_counter.count_local_in_1k() }
}
