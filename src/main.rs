#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod constants;
mod enums;
mod minesweeper;
mod state;
mod toggle;

use anyhow::Result;
use enums::*;
use minesweeper::*;
use state::*;
use std::process;
use toggle::*;

use eframe::{egui, glow, Theme};
use egui::{Color32, Key, KeyboardShortcut, Modifiers, Stroke, Vec2, ViewportCommand};
use egui_extras::install_image_loaders;
use itertools::iproduct;
use std::time::SystemTime;

fn now() -> f64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs_f64(),
        Err(_) => 0.0,
    }
}

#[derive(Clone)]
struct MinesOfRustApp {
    gameboard: GameBoard,
    state: AppState,
    image_loaders_installed: bool,
    detonated_on: Option<Coordinate>,
    game_state: GameState,
    game_started: f64,
    game_finished: f64,
    game_settings: GameSettings,
}

fn main() -> Result<(), eframe::Error> {
    let state = AppState::load_from_userhome().unwrap_or_else(|_| AppState::default());
    let settings = GameSettings::settings_for_difficulty(&state.difficulty);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(Vec2::new(
                settings.ui_width,
                settings.ui_height,
            ))
            .with_resizable(true),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        default_theme: if state.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        },
        ..Default::default()
    };

    let app = Box::new(MinesOfRustApp {
        gameboard: GameBoard::new(
            settings.width,
            settings.height,
        ),
        state,
        image_loaders_installed: false,
        detonated_on: None,
        game_state: GameState::NotStarted,
        game_started: 0.0,
        game_finished: 0.0,
        game_settings: settings
    });

    eframe::run_native("Mines of Rust", options, Box::new(|_cc| app))
}

// https://github.com/emilk/egui/discussions/1574
pub(crate) fn load_icon() -> egui::IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../assets/mine.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    egui::IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

impl eframe::App for MinesOfRustApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.on_update(ctx, frame).expect("Failed to update UI");
    }

    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.state.save_to_userhome();
    }
}

impl MinesOfRustApp {
    fn update_difficulty_settings(&mut self) {
        self.game_settings = match self.state.difficulty {
            GameDifficulty::Beginner => GameSettings::beginner(),
            GameDifficulty::Intermediate => GameSettings::intermediate(),
            GameDifficulty::Expert => GameSettings::expert(),
            // _ => unimplemented!(),
        };
    }

    fn reset_new_game(&mut self, ctx: &egui::Context) -> Result<(), Error> {
        self.gameboard = GameBoard::new(
            self.game_settings.width,
            self.game_settings.height,
        );
        self.game_state = GameState::NotStarted;
        self.detonated_on = None;
        self.game_started = now();

        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2 {
            x: self.game_settings.ui_width,
            y: self.game_settings.ui_height,
        }));

        Ok(())
    }

    fn reset_existing_game(&mut self, _ctx: &egui::Context) -> Result<(), Error> {
        self.gameboard.reset_existing();

        self.game_state = GameState::NotStarted;
        self.game_started = now();

        Ok(())
    }

    fn start_game(&mut self, first_click: Coordinate) -> Result<(), Error> {
        println!(
            "Starting game with fist click at x={}, y={}",
            first_click.x, first_click.y
        );

        // Make sure we remove any previous mines
        //self.gameboard.reset();
        if !self.gameboard.is_populated {
            self.gameboard
                .populate_mines_around(self.game_settings.num_mines, Some(first_click))?;
        }

        self.game_started = now();
        self.game_state = GameState::Playing;

        if self.game_settings.use_numerals {
            self.gameboard.populate_numerals()?;
        }

        Ok(())
    }

    fn on_update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<(), Error> {
        if !self.image_loaders_installed {
            install_image_loaders(ctx);
            self.image_loaders_installed = true;
        }
        // println!(
        //     "width: {}, height: {}",
        //     ctx.available_rect().width(),
        //     ctx.available_rect().height()
        // );

        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .min_height(50.0)
            .show(ctx, |ui| {
                self.state.dark_mode = ui.visuals().dark_mode; // I don't like having this here.

                if ui.input_mut(|i| {
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::N))
                }) {
                    println!("ctrl+n is pressed, resetting game");
                    self.reset_new_game(ctx).expect("Error building new game");
                }
                if ui.input_mut(|i| {
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::R))
                }) {
                    println!("ctrl+r is pressed, resetting existing game");
                    self.reset_existing_game(ctx)
                        .expect("Error rebuilding game");
                }
                if ui.input_mut(|i| {
                    i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Q))
                }) {
                    println!("Boss can see screen. Ctrl+q is pressed, exiting");
                    process::exit(0);
                }
                ui.vertical_centered(|ui| {
                    if self.face_ui(ui).clicked() {
                        self.reset_new_game(ctx).expect("Error building new game");
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if self.game_state != GameState::Paused {
                    self.game_board_ui(ui, !self.game_state.game_ended());
                } else {
                    self.game_board_paused_ui(ui);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(false)
            .min_height(50.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    egui::Grid::new("app_options")
                        .num_columns(2)
                        .spacing([10.0, 50.0])
                        .striped(false)
                        .show(ui, |ui| {
                            //egui::CollapsingHeader::new("Options")
                            //    .default_open(false)
                            //    .show(ui, |ui| {
                            self.options_ui(ctx, ui);
                            //    });
                            self.status_ui(ui);
                        });
                });
            });
        if self.game_state == GameState::Playing {
            ctx.request_repaint();
        }
        Ok(())
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("");
            ui.heading(format!(
                "{} of {}",
                self.gameboard.num_flags(),
                self.gameboard.num_mines
            ));

            let s = if self.game_state == GameState::Playing && self.gameboard.is_loss_configuration()
            {
                self.game_state = GameState::EndedLoss;
                self.game_finished = now();
                "".to_string()
            } else if self.game_state == GameState::Playing
                && self.gameboard.is_win_configuration()
            {
                self.game_state = GameState::EndedWin;
                self.gameboard.flag_all_mines();
                self.game_finished = now();
                "".to_string()
            } else if self.game_state == GameState::Playing {
                format!("Time: {:.2}", now() - self.game_started)
            } else if self.game_state == GameState::Paused {
                format!("Time: {:.2}", self.game_started)
            } else if self.game_state.game_ended() {
                format!(
                    "Time: {:.2}",
                    self.game_finished - self.game_started
                )
            } else {
                "".to_string()
            };
            ui.heading(s);

            if self.game_state == GameState::Playing {
                if ui.button("Pause").clicked() {
                    self.pause_game();
                }
            } else if self.game_state == GameState::Paused {
                if ui.button("Resume").clicked() {
                    self.resume_game();
                }
            }
        });
    }

    fn pause_game(&mut self) {
        self.game_state = GameState::Paused;
        self.game_started = now() - self.game_started;
    }

    fn resume_game(&mut self) {
        self.game_state = GameState::Playing;
        self.game_started = now() - self.game_started;
    }


    fn options_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Grid::new("app_options")
            .num_columns(2)
            .spacing([5.0, 5.0])
            .min_row_height(30.0)
            .striped(false)
            .show(ui, |ui| {
                ui.label("Difficulty:");

                let cb = egui::ComboBox::new("Cartesian axis", "")
                    .width(0_f32)
                    .selected_text(self.state.difficulty.as_str());
                cb.show_ui(ui, |ui| {
                    let b = ui.selectable_value(
                        &mut self.state.difficulty,
                        GameDifficulty::Beginner,
                        "Beginner",
                    );
                    let i = ui.selectable_value(
                        &mut self.state.difficulty,
                        GameDifficulty::Intermediate,
                        "Intermediate",
                    );
                    let e = ui.selectable_value(
                        &mut self.state.difficulty,
                        GameDifficulty::Expert,
                        "Expert",
                    );
                    // I don't like this pattern:
                    if b.changed() || i.changed() || e.changed() {
                        self.update_difficulty_settings();
                        self.reset_new_game(ctx).expect("Failed to reset game");
                    }
                });
                ui.end_row();

                ui.label("Left Click Chords:");
                toggle_ui(ui, &mut self.state.left_click_chord);
                ui.end_row();

                ui.label("Light/Dark Mode:");
                egui::widgets::global_dark_light_mode_switch(ui);
            });
    }

    /// Returns the first found Explosion in a list of cascaded play results
    fn first_losing_square_of_vec(play_result:&[PlayResult]) -> Option<Coordinate> {
        for r in play_result {
            match r {
                PlayResult::Explosion(c) => return Some(c.clone()),
                _ => {}
            };
        }
        None
    }

    /// Returns the first found Explosion in either an explicit explosion or a cascaded play result
    fn first_losing_square(play_result:&PlayResult) -> Option<Coordinate> {
        match play_result {
            PlayResult::Explosion(c) => Some(c.clone()),
            PlayResult::CascadedReveal(r) => MinesOfRustApp::first_losing_square_of_vec(&r),
            _ => None
        }
    }

    fn game_board_paused_ui(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(self.game_settings.width as f32, self.game_settings.height as f32);
        let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        let revealed_color =  Color32::GRAY;
        let border_color = Color32::DARK_GRAY;

        ui.painter().rect(
            rect,
            1.0,
            revealed_color,
            Stroke::new(1.0, border_color),
        );
    }

    fn game_board_ui(&mut self, ui: &mut egui::Ui, active: bool) {
        egui::Grid::new("process_grid_outputs")
            .spacing([0.0, 0.0])
            .striped(false)
            .show(ui, |ui| {
                iproduct!(0..self.gameboard.height, 0..self.gameboard.width).for_each(|(y, x)| {
                    let sqr = self
                        .gameboard
                        .get_square(x, y)
                        .expect("Error retrieving square");

                    let detonated = if let Some(c) = &self.detonated_on {
                        c.matches(x, y)
                    } else {
                        false
                    };

                    let resp = self.square_ui(ui, &sqr, active, detonated);
                    if resp.clicked() && self.game_state == GameState::NotStarted {
                        self.start_game(Coordinate { x, y })
                            .expect("Error starting game");
                    }

                    let play_type = if active
                        && resp.clicked_by(egui::PointerButton::Primary)
                        && !self.state.left_click_chord
                    {
                        Some(RevealType::Reveal)
                    } else if active
                        && resp.clicked_by(egui::PointerButton::Primary)
                        && self.state.left_click_chord
                    {
                        Some(RevealType::RevealChord)
                    } else if  active && resp.clicked_by(egui::PointerButton::Middle) {
                        Some(RevealType::Chord)
                    } else if resp.clicked_by(egui::PointerButton::Secondary) && active {
                        Some(RevealType::Flag)
                    } else {
                        None
                    };

                    if let Some(p) = play_type {
                        if let Some(c) = MinesOfRustApp::first_losing_square(&self.gameboard.play(x, y, p).expect("Failed to play desired move")) {
                            println!("Detonated on {:?}", c);
                            self.detonated_on = Some(c.clone());
                        }
                    }


                    if x == self.gameboard.width - 1 {
                        ui.end_row();
                    }
                });
            });
    }

    fn face_ui(&self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(1.4, 1.4);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if self.game_state == GameState::EndedLoss {
            egui::Image::new(egui::include_image!("../assets/loss.png")).paint_at(ui, rect);
        } else if self.game_state == GameState::EndedWin {
            egui::Image::new(egui::include_image!("../assets/win.png")).paint_at(ui, rect);
        } else {
            egui::Image::new(egui::include_image!("../assets/happy.png")).paint_at(ui, rect);
        }

        response
    }

    fn square_ui(&self, ui: &mut egui::Ui, sqr: &Square, active:bool, is_detonated:bool) -> egui::Response {
        let desired_size = (ui.spacing().interact_size.x) * egui::vec2(1.0, 1.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        let unrevealed_color = if active && response.clicked() { Color32::WHITE } else { Color32::LIGHT_BLUE };
        let revealed_color = if is_detonated { Color32::GOLD } else { Color32::GRAY };
        let border_color = Color32::DARK_GRAY ;

        ui.painter().rect(
            rect,
            1.0,
            revealed_color,
            Stroke::new(1.0, border_color),
        );

        // Note: These are insufficient.
        // Playing
        //      Unrevealed
        //      Unrevealed Flagged
        //      Revealed numeral
        //      Revealed blank
        //      Unrevealed, Mouse down, left button
        //      Unrevealed, Mouse down, chord
        // Loss
        //      Unrevealed
        //      Unrevealed non-mined flagged
        //      Unrevealed mined flagged
        //      Revealed mined (losing play)
        //      Revealed mined (adjacent to losing play)
        //      Revealed numeral
        //      Revealed blank
        // Win
        //      Unrevealed
        //      Unrevealed flagged
        //      Revealed numeral
        //      Revealed blank
        if sqr.is_mine() && (sqr.is_revealed || self.game_state == GameState::EndedLoss) {
            egui::Image::new(egui::include_image!("../assets/mine.png")).paint_at(ui, rect);
        } else if sqr.is_flagged {
            ui.painter().rect(
                rect,
                1.0,
                unrevealed_color,
                Stroke::new(1.0, border_color),
            );
            egui::Image::new(egui::include_image!("../assets/flag.png")).paint_at(ui, rect);
        } else if sqr.is_revealed {
            match sqr.numeral {
                1 => egui::Image::new(egui::include_image!("../assets/1.png")).paint_at(ui, rect),
                2 => egui::Image::new(egui::include_image!("../assets/2.png")).paint_at(ui, rect),
                3 => egui::Image::new(egui::include_image!("../assets/3.png")).paint_at(ui, rect),
                4 => egui::Image::new(egui::include_image!("../assets/4.png")).paint_at(ui, rect),
                5 => egui::Image::new(egui::include_image!("../assets/5.png")).paint_at(ui, rect),
                6 => egui::Image::new(egui::include_image!("../assets/6.png")).paint_at(ui, rect),
                7 => egui::Image::new(egui::include_image!("../assets/7.png")).paint_at(ui, rect),
                8 => egui::Image::new(egui::include_image!("../assets/8.png")).paint_at(ui, rect),
                _ => {}
            };
        } else {
            ui.painter().rect(
                rect,
                1.0,
                unrevealed_color,
                Stroke::new(1.0, border_color),
            );
        }

        response
    }
}
