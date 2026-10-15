use macroquad::rand::gen_range;

use crate::{
    agent::{Action, Direction},
    observation::Observation,
    world::Tile,
};

/// Anything that can decide what an agent does next.
pub trait Brain {
    fn decide(&mut self, obs: &Observation) -> Action;
}

/// A smooth brain: random walk, eat if standing on a berry.
pub struct SmoothBrain;

impl Brain for SmoothBrain {
    fn decide(&mut self, obs: &Observation) -> Action {
        if obs.standing_on == Tile::Berry {
            return Action::Eat;
        }
        let dir = Direction::ALL[gen_range(0, Direction::ALL.len())];
        Action::Move(dir)
    }
}
