#[cfg(test)]
mod test_opening {
    use rusty_renju::notation::pos::pos_unchecked;
    use rusty_renju::opening::opening_agent;
    use rusty_renju::opening::opening_agent::{OpeningKind, OpeningStage};

    #[test]
    fn soosyrv8_opening() {
        let agent = opening_agent::new_agent(OpeningKind::Soosyrv8);
        let OpeningStage::Move(agent) = agent else { panic!() };

        let agent = agent.set(pos_unchecked("h8")).unwrap();
        let OpeningStage::Move(agent) = agent else { panic!() };

        let agent = agent.set(pos_unchecked("g7")).unwrap();
        let OpeningStage::Move(agent) = agent else { panic!() };

        let agent = agent.set(pos_unchecked("j7")).unwrap();
        let OpeningStage::Move(agent) = agent else { panic!() };

        let agent = agent.set(pos_unchecked("h9")).unwrap();
        let OpeningStage::Swap(agent) = agent else { panic!() };
    }

    #[test]
    fn taraguchi10_opening() {
        let agent = opening_agent::new_agent(OpeningKind::Taraguchi10);

        let OpeningStage::Move(agent) = agent else { panic!() };
    }

    #[test]
    fn random4_opening() {
        let agent = opening_agent::new_agent(OpeningKind::Random4);
    }

}
