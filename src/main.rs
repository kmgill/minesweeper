mod constants;
mod enums;
mod minesweeper;
mod state;
mod toggle;

use enums::*;
use minesweeper::*;
use state::*;
use toggle::*;

use anyhow::Result;

use eframe::{egui, glow};
use egui::{Color32, Stroke, Vec2, ViewportCommand};
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
    gameboard: minesweeper::GameBoard,
    state: AppState,
    image_loaders_installed: bool,
}

fn main() -> Result<(), eframe::Error> {
    let state = match AppState::load_from_userhome() {
        Ok(s) => s,
        Err(_) => AppState::default(),
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(Vec2::new(
                state.game_settings.ui_width,
                state.game_settings.ui_height,
            ))
            .with_resizable(true),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };

    let app = Box::new(MinesOfRustApp {
        gameboard: minesweeper::GameBoard::new_populated_around(
            state.game_settings.width,
            state.game_settings.height,
            state.game_settings.num_mines,
            Coordinate { x: 5, y: 0 },
        )
        .expect("Failed to generate a game board"),
        state: state,
        image_loaders_installed: false,
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
    fn on_exit(&mut self, _gl: Option<&glow::Context>) {
        self.state.save_to_userhome();
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.on_update(ctx, frame).expect("Failed to update UI");
    }
}

impl MinesOfRustApp {
    fn update_difficulty_settings(&mut self) {
        self.state.game_settings = match self.state.difficulty {
            GameDifficulty::Beginner => GameSettings::beginner(),
            GameDifficulty::Intermediate => GameSettings::intermediate(),
            GameDifficulty::Expert => GameSettings::expert(),
            // _ => unimplemented!(),
        };
    }

    fn reset_game(&mut self, ctx: &egui::Context) -> Result<(), Error> {
        self.gameboard = minesweeper::GameBoard::new(
            self.state.game_settings.width,
            self.state.game_settings.height,
        );
        self.state.game_state = GameState::NotStarted;
        self.state.game_started = now();

        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2 {
            x: self.state.game_settings.ui_width,
            y: self.state.game_settings.ui_height,
        }));

        Ok(())
    }

    fn start_game(&mut self, first_click: Coordinate) -> Result<(), Error> {
        println!(
            "Starting game with fist click at x={}, y={}",
            first_click.x, first_click.y
        );

        // Make sure we remove any previous mines
        self.gameboard.reset();
        self.gameboard
            .populate_mines_around(self.state.game_settings.num_mines, Some(first_click))?;

        self.state.game_started = now();
        self.state.game_state = GameState::Playing;

        if self.state.game_settings.use_numerals {
            self.gameboard.populate_numerals()?;
        }

        Ok(())
    }

    fn on_update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<(), Error> {
        if !self.image_loaders_installed {
            install_image_loaders(ctx);
            self.image_loaders_installed = true;
        }
        println!(
            "width: {}, height: {}",
            ctx.available_rect().width(),
            ctx.available_rect().height()
        );

        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .min_height(50.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if self.face_ui(ui).clicked() {
                        self.reset_game(ctx).expect("Error building new game");
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                self.game_board_ui(ui, !self.state.game_state.game_ended());
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
                            self.options_ui(ctx, ui);
                            self.status_ui(ui);
                        });
                });
            });

        ctx.request_repaint();
        Ok(())
    }

    fn status_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_centered(|ui| {
            ui.label(format!(
                "{} of {}",
                self.gameboard.num_flags(),
                self.gameboard.num_mines
            ));

            if self.state.game_state == GameState::Playing && self.gameboard.is_loss_configuration()
            {
                self.state.game_state = GameState::EndedLoss;
                self.state.game_finished = now();
            } else if self.state.game_state == GameState::Playing
                && self.gameboard.is_win_configuration()
            {
                self.state.game_state = GameState::EndedWin;
                self.gameboard.flag_all_mines();
                self.state.game_finished = now();
            } else if self.state.game_state == GameState::Playing {
                ui.label(format!("Time: {:.2}", now() - self.state.game_started));
            } else if self.state.game_state.game_ended() {
                ui.label(format!(
                    "Time: {:.2}",
                    self.state.game_finished - self.state.game_started
                ));
            }
        });
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
                        self.reset_game(ctx).expect("Failed to reset game");
                    }
                });
                ui.end_row();

                ui.label("Left Click Chords:");
                toggle_ui(ui, &mut self.state.left_click_chord);
                ui.end_row();
            });
    }

    fn game_board_ui(&mut self, ui: &mut egui::Ui, active: bool) {
        egui::Grid::new("process_grid_outputs")
            .num_columns(10)
            .spacing([0.0, 0.0])
            .striped(true)
            .show(ui, |ui| {
                iproduct!(0..self.gameboard.height, 0..self.gameboard.width).for_each(|(y, x)| {
                    let sqr = self
                        .gameboard
                        .get_square(x, y)
                        .expect("Error retrieving square");

                    let resp = self.square_ui(ui, &sqr);
                    if resp.clicked() && self.state.game_state == GameState::NotStarted {
                        self.start_game(Coordinate { x, y })
                            .expect("Error starting game");
                    }

                    if active
                        && resp.clicked_by(egui::PointerButton::Primary)
                        && !self.state.left_click_chord
                    {
                        println!("Left Clicked x={}, y={} (Reveal)", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Reveal)
                            .expect("Failed to play square");
                    } else if active
                        && resp.clicked_by(egui::PointerButton::Primary)
                        && self.state.left_click_chord
                    {
                        println!("Left Clicked x={}, y={} (Chord)", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Reveal)
                            .expect("Failed to play square");
                        self.gameboard
                            .play(x, y, RevealType::Chord)
                            .expect("Failed to play square");
                    } else if resp.clicked_by(egui::PointerButton::Secondary) && active {
                        println!("Right Clicked x={}, y={} (Flag)", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Flag)
                            .expect("Failed to play square");
                    } else if active && resp.clicked_by(egui::PointerButton::Middle) {
                        println!("Middle Clicked x={}, y={} (Chord)", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Chord)
                            .expect("Failed to play square");
                    }
                    if x == self.gameboard.width - 1 {
                        ui.end_row();
                    }
                });
            });
    }

    fn face_ui(&self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(1.0, 1.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if self.state.game_state == GameState::EndedLoss {
            egui::Image::new(egui::include_image!("../assets/loss.png")).paint_at(ui, rect);
        } else if self.state.game_state == GameState::EndedWin {
            egui::Image::new(egui::include_image!("../assets/win.png")).paint_at(ui, rect);
        } else {
            egui::Image::new(egui::include_image!("../assets/happy.png")).paint_at(ui, rect);
        }

        response
    }

    fn square_ui(&self, ui: &mut egui::Ui, sqr: &minesweeper::Square) -> egui::Response {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(1.0, 1.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        ui.painter().rect(
            rect,
            1.0,
            Color32::GRAY,
            Stroke::new(2.0, Color32::DARK_GRAY),
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
        if sqr.is_mine() && (sqr.is_revealed || self.state.game_state == GameState::EndedLoss) {
            egui::Image::new(egui::include_image!("../assets/mine.png")).paint_at(ui, rect);
        } else if sqr.is_flagged {
            ui.painter().rect(
                rect,
                1.0,
                Color32::LIGHT_BLUE,
                Stroke::new(2.0, Color32::DARK_GRAY),
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
                Color32::LIGHT_BLUE,
                Stroke::new(2.0, Color32::DARK_GRAY),
            );
        }

        response
    }
}
