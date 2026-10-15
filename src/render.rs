use macroquad::{
    color::Color,
    math::{Vec2, vec2},
    shapes::{draw_circle, draw_rectangle},
};

use crate::agent::{Agent, MAX_ENERGY};
use crate::world::{Pos, Tile, World};

// --- palette ---
pub const COLOR_BACKGROUND: Color = Color::new(0.06, 0.07, 0.08, 1.0);
const COLOR_EMPTY: Color = Color::new(0.11, 0.13, 0.11, 1.0);
const COLOR_WALL: Color = Color::new(0.28, 0.26, 0.24, 1.0);
const COLOR_BERRY: Color = Color::new(0.13, 0.18, 0.13, 1.0);
const COLOR_BERRY_DOT: Color = Color::new(0.85, 0.25, 0.40, 1.0);
const COLOR_AGENT: Color = Color::new(0.45, 0.78, 0.95, 1.0);
const COLOR_AGENT_DEAD: Color = Color::new(0.40, 0.38, 0.42, 1.0);
const COLOR_ENERGY: Color = Color::new(0.35, 0.85, 0.45, 1.0);
const COLOR_ENERGY_BG: Color = Color::new(0.0, 0.0, 0.0, 0.55);

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub origin: Vec2,
    pub tile_size: f32,
}

impl Layout {
    /// Fit the grid to the a given area
    pub fn fit(world: &World, area_w: f32, area_h: f32) -> Self {
        const MARGIN: f32 = 16.0;

        let tile_size = ((area_w - MARGIN * 2.0) / world.width as f32)
            .min((area_h - MARGIN * 2.0) / world.height as f32);

        let grid_w = tile_size * world.width as f32;
        let grid_h = tile_size * world.height as f32;

        Layout {
            origin: vec2((area_w - grid_w) / 2.0, (area_h - grid_h) / 2.0),
            tile_size,
        }
    }

    /// Screen rect `(x, y, w, h)` of one cell.
    pub fn tile_rect(&self, p: Pos) -> (f32, f32, f32, f32) {
        (
            self.origin.x + p.x as f32 * self.tile_size,
            self.origin.y + p.y as f32 * self.tile_size,
            self.tile_size,
            self.tile_size,
        )
    }
}

pub fn draw_world(world: &World, layout: &Layout) {
    for y in 0..world.height {
        for x in 0..world.width {
            let p = Pos::new(x, y);
            let tile = world
                .get(&p)
                .expect("x and y came from the world's own size");
            let (rx, ry, w, h) = layout.tile_rect(p);

            let color = match tile {
                Tile::Empty => COLOR_EMPTY,
                Tile::Wall => COLOR_WALL,
                Tile::Berry => COLOR_BERRY,
            };

            // Inset by 1px so the grid reads as cells, not a flat field.
            draw_rectangle(rx + 1.0, ry + 1.0, w - 2.0, h - 2.0, color);

            if tile == Tile::Berry {
                draw_circle(rx + w / 2.0, ry + h / 2.0, w * 0.18, COLOR_BERRY_DOT);
            }
        }
    }
}

pub fn draw_agents(agents: &[Agent], layout: &Layout) {
    for agent in agents {
        let (rx, ry, w, h) = layout.tile_rect(agent.pos);

        let color = if agent.alive { COLOR_AGENT } else { COLOR_AGENT_DEAD };
        draw_circle(rx + w / 2.0, ry + h / 2.0, w * 0.34, color);

        if agent.alive {
            // Energy bar across the bottom of the cell.
            let frac = (agent.energy as f32 / MAX_ENERGY as f32).clamp(0.0, 1.0);
            let bar_h = (h * 0.12).max(2.0);
            let bar_y = ry + h - bar_h - 1.0;
            draw_rectangle(rx + 1.0, bar_y, w - 2.0, bar_h, COLOR_ENERGY_BG);
            draw_rectangle(rx + 1.0, bar_y, (w - 2.0) * frac, bar_h, COLOR_ENERGY);
        }
    }
}
