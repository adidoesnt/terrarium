use std::fmt;

use crate::{
    agent::{Agent, MAX_ENERGY},
    world::{Pos, Tile, World},
};

/// How far an agent can see in each direction.
pub const VISION_RADIUS: i32 = 4;

/// Everything one agent knows at one instant.
#[derive(Clone, Debug)]
pub struct Observation {
    pub tick: u64,
    name: String,
    pub energy: i32,
    pub standing_on: Tile,
    /// Square window, side `view_side`, row-major, agent at the centre.
    pub view: Vec<char>,
    pub view_side: i32,
    /// Offsets relative to the agent. East is +x, south is +y.
    pub berries: Vec<(i32, i32)>,
    pub agents: Vec<(String, i32, i32)>,
}

impl Observation {
    pub fn new(world: &World, me: &Agent, everyone: &[Agent]) -> Self {
        let side = VISION_RADIUS * 2 + 1;
        let mut view = Vec::with_capacity((side * side) as usize);
        let mut berries = Vec::new();
        let mut agents = Vec::new();

        for dy in -VISION_RADIUS..=VISION_RADIUS {
            for dx in -VISION_RADIUS..=VISION_RADIUS {
                let p = Pos::new(me.pos.x + dx, me.pos.y + dy);

                // An agent standing here outranks the terrain underneath.
                let occupant = everyone
                    .iter()
                    .find(|a| a.alive && a.id != me.id && a.pos == p);

                let ch = if dx == 0 && dy == 0 {
                    '@'
                } else if let Some(other) = occupant {
                    agents.push((other.name.clone(), dx, dy));
                    other.name.chars().next().unwrap_or('?')
                } else {
                    match world.get(&p) {
                        Some(Tile::Empty) => '.',
                        Some(Tile::Wall) => '#',
                        Some(Tile::Berry) => {
                            berries.push((dx, dy));
                            'b'
                        }
                        None => ' ', // outside the world
                    }
                };

                view.push(ch);
            }
        }

        Observation {
            tick: world.tick,
            name: me.name.clone(),
            energy: me.energy,
            standing_on: world.get(&me.pos).unwrap_or(Tile::Empty),
            view,
            view_side: side,
            berries,
            agents,
        }
    }
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Tick {}. You are {}.", self.tick, self.name)?;
        writeln!(
            f,
            "Energy {}/{}. You lose 1 every tick. At 0 you die.",
            self.energy, MAX_ENERGY
        )?;

        let ground = match self.standing_on {
            Tile::Berry => "a berry — you can eat it right now",
            Tile::Empty => "empty ground",
            Tile::Wall => "a wall (this should be impossible)",
        };
        writeln!(f, "You are standing on {ground}.")?;
        writeln!(f)?;
        writeln!(
            f,
            "You can see {VISION_RADIUS} tiles in each direction. North is up."
        )?;
        writeln!(f)?;

        for row in 0..self.view_side {
            let start = (row * self.view_side) as usize;
            let end = start + self.view_side as usize;
            let line: String = self.view[start..end].iter().collect();
            writeln!(f, "  {line}")?;
        }

        writeln!(f)?;
        writeln!(
            f,
            "  @ you  . empty  # wall  b berry  A-Z agent  (blank) off-map"
        )?;
        writeln!(f)?;

        if self.berries.is_empty() {
            writeln!(f, "No berries in sight.")?;
        } else {
            writeln!(f, "Berries you can see:")?;
            for (dx, dy) in &self.berries {
                writeln!(f, "  - {}", describe_offset(*dx, *dy))?;
            }
        }

        if !self.agents.is_empty() {
            writeln!(f, "Other agents you can see:")?;
            for (name, dx, dy) in &self.agents {
                writeln!(f, "  - {name}, {}", describe_offset(*dx, *dy))?;
            }
        }

        Ok(())
    }
}

/// "3 east, 1 north" — a direction an agent can act on without doing arithmetic.
fn describe_offset(dx: i32, dy: i32) -> String {
    let mut parts = Vec::new();
    if dx != 0 {
        parts.push(format!(
            "{} {}",
            dx.abs(),
            if dx > 0 { "east" } else { "west" }
        ));
    }
    if dy != 0 {
        parts.push(format!(
            "{} {}",
            dy.abs(),
            if dy > 0 { "south" } else { "north" }
        ));
    }
    if parts.is_empty() {
        "right here".to_string()
    } else {
        parts.join(", ")
    }
}
