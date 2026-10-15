use crate::world::Pos;

pub const MAX_ENERGY: i32 = 150;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
    ];

    /// Screen-space delta. North is -1 on y because y grows downward.
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
        }
    }
}

/// Everything an agent is allowed to do in one tick. Exactly one.
/// Capture the intent so that the action can be decided upon and executed separately.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Move(Direction),
    Eat,
    Wait,
}

pub struct Agent {
    pub id: usize,
    pub name: String,
    pub pos: Pos,
    pub energy: i32,
    pub alive: bool,
}

impl Agent {
    pub fn new(id: usize, name: &str, pos: Pos) -> Self {
        Agent {
            id,
            name: name.to_string(),
            pos,
            energy: 100,
            alive: true,
        }
    }
}
