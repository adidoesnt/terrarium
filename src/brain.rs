use macroquad::rand::gen_range;

use crate::{
    agent::{Action, Agent, Direction},
    world::{Tile, World},
};

/// Anything that can decide what an agent does next.
pub trait Brain {
    fn decide(&mut self, world: &World, me: &Agent) -> Action;
}

/// A smooth brain: random walk, eat if standing on a berry.
pub struct SmoothBrain;

impl Brain for SmoothBrain {
    fn decide(&mut self, world: &World, me: &Agent) -> Action {
        if world.get(&me.pos) == Some(Tile::Berry) {
            return Action::Eat;
        }
        let dir = Direction::ALL[gen_range(0, Direction::ALL.len())];
        Action::Move(dir)
    }
}
