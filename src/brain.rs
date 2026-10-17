use macroquad::rand::gen_range;

use crate::{
    agent::{Action, Direction},
    observation::Observation,
    world::Tile,
};

/// Anything that can decide what an agent does next.
pub trait Brain {
    /// `None` means "still thinking, ask me again".
    /// The `obs` passed by sim will not change until
    /// we return `Some`
    fn decide(&mut self, obs: &Observation) -> Option<Action>;
}

/// A smooth brain: random walk, eat if standing on a berry.
pub struct SmoothBrain;

impl Brain for SmoothBrain {
    fn decide(&mut self, obs: &Observation) -> Option<Action> {
        if obs.standing_on == Tile::Berry {
            return Some(Action::Eat);
        }
        let dir = Direction::ALL[gen_range(0, Direction::ALL.len())];
        Some(Action::Move(dir))
    }
}

/// A LLM-based brain: uses an LLM to make decisions based on observations.
pub struct LlmBrain;

impl Brain for LlmBrain {
    fn decide(&mut self, _obs: &Observation) -> Option<Action> {
        None
    }
}
