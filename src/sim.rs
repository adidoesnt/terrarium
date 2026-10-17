use crate::agent::{Action, Agent, MAX_ENERGY};
use crate::brain::{Brain, LlmBrain, SmoothBrain};
use crate::observation::Observation;
use crate::world::{Pos, Tile, World};

const ENERGY_PER_BERRY: i32 = 25;
const ENERGY_PER_TICK: i32 = 1;

pub struct Sim {
    pub world: World,
    pub agents: Vec<Agent>,
    brains: Vec<Box<dyn Brain>>,
    /// Phase 1 (decision making) progress, kept across frames
    /// `Some` returned once the agent's brain has decided
    pending: Vec<Option<Action>>,
}

impl Sim {
    pub fn new(width: i32, height: i32, seed: u64, agent_count: usize) -> Self {
        let world = World::new(width, height, seed);
        let mut agents = Vec::new();
        let mut brains: Vec<Box<dyn Brain>> = Vec::new();

        // TODO: Randomise later.
        let names = ["Ada", "Bo", "Cy", "Dot", "Eli"];
        for id in 0..agent_count {
            let pos = find_empty_spot(&world, &agents);
            agents.push(Agent::new(id, names[id % names.len()], pos));

            let brain: Box<dyn Brain> = if id == 0 {
                Box::new(LlmBrain::new())
            } else {
                Box::new(SmoothBrain)
            };
            brains.push(brain);
        }
        let pending = vec![None; agent_count];

        Sim {
            world,
            agents,
            brains,
            pending,
        }
    }

    pub fn observe(&self, id: usize) -> Observation {
        Observation::new(&self.world, &self.agents[id], &self.agents)
    }

    /// Phase 1 on its own: ask anyone who hasn't answered yet. The world stays frozen
    /// until everyone has, so a slow brain holds up the turn instead of losing it.
    /// Safe to call every frame (each brain is asked once per turn), which lets
    /// brains think while the clock waits. Returns whether everyone has answered.
    pub fn ask_brains(&mut self) -> bool {
        for agent in &self.agents {
            if !agent.alive || self.pending[agent.id].is_some() {
                continue;
            }
            let obs = self.observe(agent.id);
            self.pending[agent.id] = self.brains[agent.id].decide(&obs);
        }

        self.agents
            .iter()
            .all(|a| !a.alive || self.pending[a.id].is_some())
    }

    #[must_use]
    pub fn try_tick(&mut self) -> bool {
        // Phase 1: everyone decides, nobody moves.
        if !self.ask_brains() {
            return false;
        }

        // Phase 2: apply every decision, emptying the slot for next turn
        for id in 0..self.agents.len() {
            if let Some(action) = self.pending[id].take() {
                self.apply(id, action);
            }
        }

        // Phase 3: metabolism
        for agent in &mut self.agents {
            if !agent.alive {
                continue;
            }
            agent.energy -= ENERGY_PER_TICK;
            if agent.energy <= 0 {
                agent.energy = 0;
                agent.alive = false;
            }
        }

        self.world.tick += 1;
        true
    }

    fn apply(&mut self, id: usize, action: Action) {
        match action {
            Action::Wait => {}

            Action::Move(dir) => {
                let target = self.agents[id].pos.step(dir);
                let occupied = self
                    .agents
                    .iter()
                    .any(|a| a.alive && a.id != id && a.pos == target);

                if self.world.is_walkable(&target) && !occupied {
                    self.agents[id].pos = target;
                }
            }

            Action::Eat => {
                let pos = self.agents[id].pos;
                if self.world.get(&pos) == Some(Tile::Berry) {
                    self.world.set(&pos, Tile::Empty);
                    let agent = &mut self.agents[id];
                    agent.energy = (agent.energy + ENERGY_PER_BERRY).min(MAX_ENERGY);
                }
            }
        }
    }
}

fn find_empty_spot(world: &World, taken: &[Agent]) -> Pos {
    loop {
        let p = Pos::new(
            macroquad::rand::gen_range(1, world.width - 1),
            macroquad::rand::gen_range(1, world.height - 1),
        );
        let free = world.get(&p) == Some(Tile::Empty) && !taken.iter().any(|a| a.pos == p);
        if free {
            return p;
        }
    }
}
