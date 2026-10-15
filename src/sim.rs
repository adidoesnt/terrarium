use crate::agent::{Action, Agent, MAX_ENERGY};
use crate::brain::{Brain, SmoothBrain};
use crate::world::{Pos, Tile, World};

const ENERGY_PER_BERRY: i32 = 25;
const ENERGY_PER_TICK: i32 = 1;

pub struct Sim {
    pub world: World,
    pub agents: Vec<Agent>,
    brains: Vec<Box<dyn Brain>>,
}

impl Sim {
    pub fn new(width: i32, height: i32, seed: u64, agent_count: usize) -> Self {
        let world = World::new(width, height, seed);
        let mut agents = Vec::new();
        let mut brains: Vec<Box<dyn Brain>> = Vec::new();

        // Randomise later.
        let names = ["Ada", "Bo", "Cy", "Dot", "Eli"];
        for id in 0..agent_count {
            let pos = find_empty_spot(&world, &agents);
            agents.push(Agent::new(id, names[id % names.len()], pos));
            brains.push(Box::new(SmoothBrain));
        }

        Sim {
            world,
            agents,
            brains,
        }
    }

    pub fn tick(&mut self) {
        // Phase 1: everyone decides, nobody moves
        let mut decisions: Vec<(usize, Action)> = Vec::new();
        for agent in &self.agents {
            if !agent.alive {
                continue;
            }
            let action = self.brains[agent.id].decide(&self.world, agent);
            decisions.push((agent.id, action));
        }

        // Phase 2: apply every decision
        for (id, action) in decisions {
            self.apply(id, action);
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
