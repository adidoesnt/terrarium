use std::{
    sync::mpsc::{Receiver, Sender, TryRecvError, channel},
    thread,
};

use macroquad::rand::gen_range;

use crate::{
    agent::{Action, Direction},
    observation::Observation,
    ollama,
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
    pub fn new(model: &str) -> Self {
        let (to_worker, worker_rx) = channel::<Observation>();
        let (worker_tx, from_worker) = channel::<Action>();
        let client = ollama::Client::new(model);

        thread::spawn(move || {
            // Ends when every Sender<Observation> is dropped.
            for obs in worker_rx {
                let action = match client.decide(&obs) {
                    Ok(reply) => {
                        eprintln!("[t{}] {:?}: {}", obs.tick, reply.action, reply.thought);
                        reply.action.into()
                    }
                    Err(e) => {
                        // The fallback planned in 002: a broken call must look broken.
                        eprintln!("[t{}] ollama error, waiting: {e}", obs.tick);
                        Action::Wait
                    }
                };
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
