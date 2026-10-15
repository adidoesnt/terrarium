mod agent;
mod brain;
mod render;
mod sim;
mod world;

use egui_macroquad::egui::{SidePanel, Slider};
use macroquad::{
    time::get_frame_time,
    window::{Conf, clear_background, next_frame, screen_height, screen_width},
};
use render::{COLOR_BACKGROUND, Layout, draw_world};
use world::Tile;

use crate::{render::draw_agents, sim::Sim};

const PANEL_WIDTH: f32 = 260.0;
const GRID_W: i32 = 32;
const GRID_H: i32 = 22;
const AGENT_COUNT: usize = 4;

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
    let mut sim = Sim::new(GRID_W, GRID_H, seed, AGENT_COUNT);

    let mut running = false;
    let mut ticks_per_second: f32 = 4.0;
    let mut accumulator: f32 = 0.0;

    loop {
        // Simulation clock: decouple sim rate from frame rate.
        if running {
            let interval = 1.0 / ticks_per_second;
            accumulator += get_frame_time();
            while accumulator >= interval {
                sim.tick();
                accumulator -= interval;
            }
        } else {
            accumulator = 0.0;
        }

        clear_background(COLOR_BACKGROUND);

        let layout = Layout::fit(&sim.world, screen_width() - PANEL_WIDTH, screen_height());
        draw_world(&sim.world, &layout);
        draw_agents(&sim.agents, &layout);

        egui_macroquad::ui(|ctx| {
            SidePanel::right("inspector")
                .exact_width(PANEL_WIDTH)
                .show(ctx, |ui| {
                    ui.heading("Terrarium");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button(if running { "pause" } else { "run" }).clicked() {
                            running = !running;
                        }
                        if ui.button("step").clicked() {
                            sim.tick();
                        }
                    });
                    ui.add(Slider::new(&mut ticks_per_second, 1.0..=30.0).text("ticks/s"));
                    ui.separator();

                    ui.label(format!("tick:    {}", sim.world.tick));
                    ui.label(format!("berries: {}", sim.world.count(Tile::Berry)));
                    ui.separator();

                    ui.heading("agents");
                    for a in &sim.agents {
                        if a.alive {
                            ui.label(format!(
                                "{:<4} {:>4}  ({:>2},{:>2})",
                                a.name, a.energy, a.pos.x, a.pos.y
                            ));
                        } else {
                            ui.weak(format!("{:<4}  dead", a.name));
                        }
                    }
                    ui.separator();

                    if ui.button("regenerate").clicked() {
                        seed += 1;
                        sim = Sim::new(GRID_W, GRID_H, seed, AGENT_COUNT);
                        running = false;
                    }
                });
        });
        egui_macroquad::draw();

        next_frame().await
    }
}
