mod agent;
mod brain;
mod render;
mod world;

use egui_macroquad::egui::SidePanel;
use macroquad::window::{Conf, clear_background, next_frame, screen_height, screen_width};
use render::{COLOR_BACKGROUND, Layout, draw_world};
use world::{Tile, World};

const PANEL_WIDTH: f32 = 260.0;
const GRID_W: i32 = 32;
const GRID_H: i32 = 22;

fn window_conf() -> Conf {
    Conf {
        window_title: "Terrarium".to_owned(),
        window_width: 1100,
        window_height: 760,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut seed: u64 = 1;
    let mut world = World::new(GRID_W, GRID_H, seed);

    loop {
        clear_background(COLOR_BACKGROUND);

        let layout = Layout::fit(&world, screen_width() - PANEL_WIDTH, screen_height());
        draw_world(&world, &layout);

        egui_macroquad::ui(|ctx| {
            SidePanel::right("inspector")
                .exact_width(PANEL_WIDTH)
                .show(ctx, |ui| {
                    ui.heading("Terrarium");
                    ui.separator();
                    ui.label(format!("tick:    {}", world.tick));
                    ui.label(format!("size:    {} x {}", world.width, world.height));
                    ui.label(format!("berries: {}", world.count(Tile::Berry)));
                    ui.label(format!("walls:   {}", world.count(Tile::Wall)));
                    ui.separator();
                    if ui.button("regenerate").clicked() {
                        seed += 1;
                        world = World::new(GRID_W, GRID_H, seed);
                    }
                });
        });
        egui_macroquad::draw();

        next_frame().await
    }
}
