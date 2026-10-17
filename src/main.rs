mod agent;
mod brain;
mod observation;
mod ollama;
mod render;
mod sim;
mod world;

use egui_macroquad::egui::{Button, SidePanel, Slider};
use macroquad::{
    time::{get_frame_time, get_time},
    window::{Conf, clear_background, next_frame, screen_height, screen_width},
};
use render::{COLOR_BACKGROUND, Layout, draw_world};
use world::Tile;

use crate::{render::draw_agents, sim::Sim};

const PANEL_WIDTH: f32 = 340.0;
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
    let mut step_requested = false;

    let mut ticks_per_second: f32 = 4.0;
    // Measured tick rate, refreshed about once a second.
    let mut actual_rate: f64 = 0.0;
    let mut rate_since = get_time();
    let mut rate_from_tick = sim.world.tick;

    let mut accumulator: f32 = 0.0;

    // Which agents observations we are viewing.
    let mut selected: usize = 0;

    loop {
        // Let brains think between ticks, not only when one is due.
        let _ = sim.ask_brains();

        // Simulation clock: decouple sim rate from frame rate.
        if running {
            let interval = 1.0 / ticks_per_second;
            accumulator += get_frame_time();
            while accumulator >= interval {
                if !sim.try_tick() {
                    // Brains still thinking. Hold one tick ready, but don't bank the wait
                    // Avoids a burst of ticks when the LlmBrains decide.
                    accumulator = interval;
                    break;
                }
                accumulator -= interval;
            }
        } else {
            accumulator = 0.0;
        }

        if step_requested && sim.try_tick() {
            step_requested = false;
        }

        // Tick-rate measurement: after every tick source has had its turn this frame.
        let now = get_time();
        if now - rate_since >= 1.0 {
            actual_rate = (sim.world.tick - rate_from_tick) as f64 / (now - rate_since);
            rate_since = now;
            rate_from_tick = sim.world.tick;
        }

        clear_background(COLOR_BACKGROUND);

        let layout = Layout::fit(&sim.world, screen_width() - PANEL_WIDTH, screen_height());
        draw_world(&sim.world, &layout);
        draw_agents(&sim.agents, selected, &layout);

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
                        if ui.add_enabled(!running, Button::new("step")).clicked() {
                            step_requested = true;
                        }
                    });
                    ui.add(Slider::new(&mut ticks_per_second, 1.0..=30.0).text("max ticks/s"));
                    ui.label(format!("actual: {actual_rate:.1} ticks/s"));
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

                    ui.horizontal(|ui| {
                        for a in &sim.agents {
                            let label = if a.alive {
                                a.name.clone()
                            } else {
                                format!("{} †", a.name)
                            };
                            ui.selectable_value(&mut selected, a.id, label);
                        }
                    });
                    ui.collapsing("what they see", |ui| {
                        if sim.agents[selected].alive {
                            ui.monospace(sim.observe(selected).to_string());
                        } else {
                            ui.weak("dead — no observation");
                        }
                    });
                    ui.separator();

                    if ui.button("regenerate").clicked() {
                        seed += 1;
                        sim = Sim::new(GRID_W, GRID_H, seed, AGENT_COUNT);
                        running = false;
                        step_requested = false;
                        rate_from_tick = 0;
                        rate_since = get_time();
                        selected = 0;
                    }
                });
        });
        egui_macroquad::draw();

        next_frame().await
    }
}
