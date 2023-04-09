#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod process;
mod process_list;
mod watcher;

use std::env;

use crate::watcher::Pair;
use bytemuck::Pod;
use eframe::egui;
use livesplit_core::timing::formatter::{SegmentTime, TimeFormatter};
use livesplit_core::timing::TimingMethod;
use livesplit_core::{Run, Segment, Timer};
use process::Process;
use process_list::ProcessList;
use watcher::Watcher as WatcherBase;

struct State {
    process: Option<Process>,
    module: u64,
    process_list: ProcessList,
    menu_timer: Timer,
    battle_timer: Timer,
    game_state: Watcher<u8>,
}

impl State {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        cc.egui_ctx.set_pixels_per_point(2.0);

        let mut menu_run = Run::new();
        menu_run.set_game_name("Run");
        menu_run.push_segment(Segment::new("Menu Split"));
        let mut menu_timer = Timer::new(menu_run).expect("Run with at least one segment provided");
        menu_timer.set_current_timing_method(TimingMethod::RealTime);

        let mut battle_run = Run::new();
        battle_run.set_game_name("Run");
        battle_run.push_segment(Segment::new("Battle Split"));
        let mut battle_timer =
            Timer::new(battle_run).expect("Run with at least one segment provided");
        battle_timer.set_current_timing_method(TimingMethod::RealTime);

        Self {
            process: None,
            module: 0,
            game_state: Watcher::new(vec![0x4F7AB68, 0x234]),
            menu_timer,
            battle_timer,
            process_list: ProcessList::new(),
        }
    }
}

struct Watcher<T> {
    watcher: WatcherBase<T>,
    address: Vec<u64>,
}

impl<T: Pod> Watcher<T> {
    fn new(address: Vec<u64>) -> Self {
        Self {
            watcher: WatcherBase::new(),
            address,
        }
    }

    fn update(&mut self, process: &Process, module: u64) -> Option<&Pair<T>> {
        let value = process.read_pointer_path64::<T>(module, &self.address);
        self.watcher.update(value.ok())
    }
}

fn main() {
    let native_options = eframe::NativeOptions {
        initial_window_size: Some(egui::Vec2{x: 300.0, y: 300.0}),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        "Octopath 2 Practice Tool",
        native_options,
        Box::new(|cc| Box::new(State::new(cc))),
    )
    .expect("Error loading application");
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        if let Some(process) = &self.process {
            match !self.process_list.is_open(sysinfo::Pid::from(process.pid as usize)) {
                true => {
                    self.process = None;
                    return
                },
                _ => ()
            }
        };

        if let Some(process) = &self.process {
            let game_state = self.game_state.update(&process, self.module).unwrap();

            // start/stop timer when entering menu
            if game_state.old != 4 && game_state.current == 4 {
                self.menu_timer.reset(false);
                self.menu_timer.start();
            } else if game_state.old == 4 && game_state.current != 4 {
                // pause timer when leaving menu
                self.menu_timer.split();
            }

            // start/stop timer when entering battle
            if game_state.old != 6 && game_state.current == 6 {
                self.battle_timer.reset(false);
                self.battle_timer.start();
            } else if game_state.old == 6 && game_state.current != 6 {
                self.battle_timer.split();
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("some_unique_id").show(ui, |ui| {
                    // ui.label("game_state:");
                    // ui.label(game_state.current.to_string());
                    // ui.end_row();

                    ui.label("Menu:  ");
                    let formatter = SegmentTime::new();
                    let formatted = formatter.format(self.menu_timer.current_attempt_duration());
                    let menu_time = format!("{}", formatted);
                    ui.label(menu_time);
                    ui.end_row();

                    ui.label("Battle:");
                    let formatter = SegmentTime::new();
                    let formatted = formatter.format(self.battle_timer.current_attempt_duration());
                    let menu_time = format!("{}", formatted);
                    ui.label(menu_time);
                    ui.end_row();
                });
            });
        } else {
            let process_name = match env::consts::OS {
                "windows" => "Octopath_Traveler2",
                "linux" => "Octopath_Travel",
                _ => "Octopath_Travel",
            };
            match Process::with_name(process_name, &mut self.process_list) {
                Ok(mut process) => {
                    match process.module_address("Octopath_Traveler2-Win64-Shipping.exe") {
                        Ok(address) => {
                            self.process = Some(process);
                            self.module = address;
                        }
                        _ => (),
                    };
                }
                Err(_err) => (),
            }

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("Game is not running...");
            });
        }

    }
}
