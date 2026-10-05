use rusty_renju::notation::rule::RuleKind;

#[cfg(not(feature = "neural-eval"))]
pub type ActiveEvaluator<const R: RuleKind> = crate::eval::heuristic_evaluator::HeuristicEvaluator<R>;
#[cfg(feature = "neural-eval")]
pub type ActiveEvaluator<const R: RuleKind> = crate::eval::heuristic_evaluator::HeuristicEvaluator<R>;
