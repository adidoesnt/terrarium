use std::{
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread,
    time::Duration,
};

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
/// Thinks on its own thread, so a slow model never blocks the frame loop
pub struct LlmBrain {
    to_worker: Sender<Observation>,
    from_worker: Receiver<Action>,
    /// Whether a request is in-flight
    thinking: bool,
}

impl Brain for LlmBrain {
    fn decide(&mut self, obs: &Observation) -> Option<Action> {
        if !self.thinking {
            // The sim guarantees `obs` won't change until we return Some,
            // so sending it once per turn is enough.
            if self.to_worker.send(obs.clone()).is_err() {
                return Some(Action::Wait);
            }
            self.thinking = true;
        }

        match self.from_worker.try_recv() {
            Ok(action) => {
                self.thinking = false;
                Some(action)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Action::Wait),
        }
    }
}

impl LlmBrain {
    pub fn new() -> Self {
        let (to_worker, worker_rx) = channel::<Observation>();
        let (worker_tx, from_worker) = channel::<Action>();

        thread::spawn(move || {
            // Ends when every Sender<Observation> is dropped.
            for obs in worker_rx {
                let action = think(&obs);
                if worker_tx.send(action).is_err() {
                    break; // brain dropped mid-thought; nobody to answer
                }
            }
        });

        LlmBrain {
            to_worker,
            from_worker,
            thinking: false,
        }
    }
}

/// Stand-in for the model: slow, and deterministic so runs stay reproducible.
fn think(obs: &Observation) -> Action {
    thread::sleep(Duration::from_secs(1));
    if obs.standing_on == Tile::Berry {
        return Action::Eat;
    }
    Action::Move(Direction::ALL[(obs.tick % 4) as usize])
}
