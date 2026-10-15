use macroquad::rand::{gen_range, srand};

/// A coordinate in the grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    pub fn step(self, dir: crate::agent::Direction) -> Pos {
        let (dx, dy) = dir.delta();
        Pos::new(self.x + dx, self.y + dy)
    }
}

/// Occupant of a particular cell.
/// Does not cover agents.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Wall,
    Berry,
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub tick: u64,
    tiles: Vec<Tile>,
}

impl World {
    pub fn new(width: i32, height: i32, seed: u64) -> Self {
        srand(seed);

        let mut world = World {
            width,
            height,
            tick: 0,
            tiles: vec![Tile::Empty; (width * height) as usize],
        };

        // Solid border so nothing can wander off the map
        for x in 0..width {
            world.set(&Pos::new(x, 0), Tile::Wall);
            world.set(&Pos::new(x, height - 1), Tile::Wall);
        }
        for y in 0..height {
            world.set(&Pos::new(0, y), Tile::Wall);
            world.set(&Pos::new(width - 1, y), Tile::Wall);
        }

        // Scatter some random interior walls.
        for _ in 0..(width * height) / 25 {
            let p = Pos::new(gen_range(1, width - 1), gen_range(1, height - 1));
            world.set(&p, Tile::Wall);
        }
        // Scatter some random berries
        for _ in 0..(width * height) / 12 {
            let p = Pos::new(gen_range(1, width - 1), gen_range(1, height - 1));
            if world.get(&p) == Some(Tile::Empty) {
                world.set(&p, Tile::Berry);
            }
        }

        world
    }

    pub fn in_bounds(&self, p: &Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    /// Grid coords -> position in the flat `tiles` vector.
    /// `None` means "off the map", which callers must handle.
    fn index(&self, p: &Pos) -> Option<usize> {
        if self.in_bounds(p) {
            Some((p.y * self.width + p.x) as usize)
        } else {
            None
        }
    }

    pub fn get(&self, p: &Pos) -> Option<Tile> {
        self.index(p).map(|i| self.tiles[i])
    }

    pub fn set(&mut self, p: &Pos, tile: Tile) {
        if let Some(i) = self.index(p) {
            self.tiles[i] = tile;
        }
    }

    pub fn is_walkable(&self, p: &Pos) -> bool {
        matches!(self.get(p), Some(Tile::Empty) | Some(Tile::Berry))
    }

    pub fn count(&self, tile: Tile) -> usize {
        self.tiles.iter().filter(|&&t| t == tile).count()
    }
}
