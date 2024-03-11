mod minesweeper;
use std::time::Instant;

use minesweeper::*;

use anyhow::Result;

use eframe::{egui, WindowBuilder};
use egui::{Color32, Stroke, Vec2, ViewportCommand};
use egui_extras::install_image_loaders;
use itertools::iproduct;

const DEFAULT_BEGINNER_WIDTH: u32 = 9;
const DEFAULT_BEGINNER_HEIGHT: u32 = 9;
const DEFAULT_BEGINNER_NUM_MINES: u32 = 10;
const DEFAULT_BEGINNER_UI_WIDTH: f32 = 376.0;
const DEFAULT_BEGINNER_UI_HEIGHT: f32 = 458.0;

const DEFAULT_INTERMEDIATE_WIDTH: u32 = 16;
const DEFAULT_INTERMEDIATE_HEIGHT: u32 = 16;
const DEFAULT_INTERMEDIATE_NUM_MINES: u32 = 40;
const DEFAULT_INTERMEDIATE_UI_WIDTH: f32 = 655.0;
const DEFAULT_INTERMEDIATE_UI_HEIGHT: f32 = 737.0;

const DEFAULT_EXPERT_WIDTH: u32 = 30;
const DEFAULT_EXPERT_HEIGHT: u32 = 16;
const DEFAULT_EXPERT_NUM_MINES: u32 = 99;
const DEFAULT_EXPERT_UI_WIDTH: f32 = 1215.0;
const DEFAULT_EXPERT_UI_HEIGHT: f32 = 737.0;

#[derive(Eq, PartialEq, Debug)]
enum GameState {
    NotStarted,
    Playing,
    EndedLoss,
    EndedWin,
}

#[derive(Eq, PartialEq, Debug, Clone)]
enum GameDifficulty {
    Beginner,
    Intermediate,
    Expert,
    Custom,
}

impl GameDifficulty {
    fn as_str(&self) -> &'static str {
        match self {
            &GameDifficulty::Beginner => "Beginner",
            &GameDifficulty::Intermediate => "Intermediate",
            &GameDifficulty::Expert => "Expert",
            &GameDifficulty::Custom => "Custom",
        }
    }
}

struct GameSettings {
    width: u32,
    height: u32,
    num_mines: u32,
    use_numerals: bool,
    ui_width: f32,
    ui_height: f32,
}

impl GameSettings {
    pub fn beginner() -> Self {
        GameSettings {
            width: DEFAULT_BEGINNER_WIDTH,
            height: DEFAULT_BEGINNER_HEIGHT,
            num_mines: DEFAULT_BEGINNER_NUM_MINES,
            use_numerals: true,
            ui_width: DEFAULT_BEGINNER_UI_WIDTH,
            ui_height: DEFAULT_BEGINNER_UI_HEIGHT,
        }
    }

    pub fn intermediate() -> Self {
        GameSettings {
            width: DEFAULT_INTERMEDIATE_WIDTH,
            height: DEFAULT_INTERMEDIATE_HEIGHT,
            num_mines: DEFAULT_INTERMEDIATE_NUM_MINES,
            use_numerals: true,
            ui_width: DEFAULT_INTERMEDIATE_UI_WIDTH,
            ui_height: DEFAULT_INTERMEDIATE_UI_HEIGHT,
        }
    }

    pub fn expert() -> Self {
        GameSettings {
            width: DEFAULT_EXPERT_WIDTH,
            height: DEFAULT_EXPERT_HEIGHT,
            num_mines: DEFAULT_EXPERT_NUM_MINES,
            use_numerals: true,
            ui_width: DEFAULT_EXPERT_UI_WIDTH,
            ui_height: DEFAULT_EXPERT_UI_HEIGHT,
        }
    }
}

impl GameState {
    pub fn game_ended(&self) -> bool {
        *self == GameState::EndedLoss || *self == GameState::EndedWin
    }
}

struct MinesweeperFoo {
    gameboard: minesweeper::GameBoard,
    game_state: GameState,
    game_started: Instant,
    game_finished: Instant,
    game_settings: GameSettings,
    difficulty: GameDifficulty,
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(Vec2::new(
                DEFAULT_INTERMEDIATE_UI_WIDTH,
                DEFAULT_INTERMEDIATE_UI_HEIGHT,
            ))
            .with_resizable(true),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };

    let default_difficulty = GameSettings::intermediate();

    let app = Box::new(MinesweeperFoo {
        gameboard: minesweeper::GameBoard::new_populated_around(
            default_difficulty.width,
            default_difficulty.height,
            default_difficulty.num_mines,
            Coordinate { x: 5, y: 0 },
        )
        .expect("Failed to generate a game board"),
        game_state: GameState::NotStarted,
        game_started: Instant::now(),
        game_finished: Instant::now(),
        game_settings: default_difficulty,
        difficulty: GameDifficulty::Intermediate,
    });

    eframe::run_native("Minesweeper Foo", options, Box::new(|_cc| app))
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

impl eframe::App for MinesweeperFoo {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.on_update(ctx, frame).expect("Failed to update UI");
    }
}

impl MinesweeperFoo {
    fn update_difficulty_settings(&mut self) {
        self.game_settings = match self.difficulty {
            GameDifficulty::Beginner => GameSettings::beginner(),
            GameDifficulty::Intermediate => GameSettings::intermediate(),
            GameDifficulty::Expert => GameSettings::expert(),
            _ => unimplemented!(),
        };
    }

    fn reset_game(&mut self, ctx: &egui::Context) -> Result<(), Error> {
        self.gameboard =
            minesweeper::GameBoard::new(self.game_settings.width, self.game_settings.height);
        self.game_state = GameState::NotStarted;
        self.game_started = std::time::Instant::now();

        ctx.send_viewport_cmd(ViewportCommand::InnerSize(Vec2 {
            x: self.game_settings.ui_width,
            y: self.game_settings.ui_height,
        }));

        Ok(())
    }

    fn start_game(&mut self, first_click: Coordinate) -> Result<(), Error> {
        println!(
            "Starting game with fist click at x={}, y={}",
            first_click.x, first_click.y
        );

        self.gameboard
            .populate_mines_around(self.game_settings.num_mines, Some(first_click))?;

        self.game_started = Instant::now();
        self.game_state = GameState::Playing;

        if self.game_settings.use_numerals {
            self.gameboard.populate_numerals()?;
        }

        Ok(())
    }

    fn on_update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<(), Error> {
        install_image_loaders(ctx);

        println!(
            "width: {}, height: {}",
            ctx.available_rect().width(),
            ctx.available_rect().height()
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if self.face_ui(ui).clicked() {
                    self.reset_game(ctx).expect("Error building new game");
                }

                self.game_board_ui(ui, !self.game_state.game_ended());

                ui.horizontal_centered(|ui| {
                    let cb = egui::ComboBox::new("Cartesian axis", "")
                        .width(0_f32)
                        .selected_text(self.difficulty.as_str());
                    cb.show_ui(ui, |ui| {
                        let b = ui.selectable_value(
                            &mut self.difficulty,
                            GameDifficulty::Beginner,
                            "Beginner",
                        );
                        let i = ui.selectable_value(
                            &mut self.difficulty,
                            GameDifficulty::Intermediate,
                            "Intermediate",
                        );
                        let e = ui.selectable_value(
                            &mut self.difficulty,
                            GameDifficulty::Expert,
                            "Expert",
                        );
                        // I don't like this pattern:
                        if b.changed() || i.changed() || e.changed() {
                            self.update_difficulty_settings();
                            self.reset_game(ctx).expect("Failed to reset game");
                        }
                    });
                    ui.label(format!(
                        "{} of {}",
                        self.gameboard.num_flags(),
                        self.gameboard.num_mines
                    ));

                    if self.game_state == GameState::Playing
                        && self.gameboard.is_loss_configuration()
                    {
                        self.game_state = GameState::EndedLoss;
                        self.game_finished = Instant::now();
                    } else if self.game_state == GameState::Playing
                        && self.gameboard.is_win_configuration()
                    {
                        self.game_state = GameState::EndedWin;
                        self.gameboard.flag_all_mines();
                        self.game_finished = Instant::now();
                    } else if self.game_state == GameState::Playing {
                        ui.label(format!(
                            "Time: {:.2}",
                            self.game_started.elapsed().as_secs_f64()
                        ));
                    } else if self.game_state.game_ended() {
                        ui.label(format!(
                            "Time: {:.2}",
                            (self.game_finished - self.game_started).as_secs_f64()
                        ));
                    }
                })
            });
        });

        ctx.request_repaint();
        Ok(())
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
                    if resp.clicked() && self.game_state == GameState::NotStarted {
                        self.start_game(Coordinate { x, y })
                            .expect("Error starting game");
                    }

                    if resp.clicked_by(egui::PointerButton::Primary) && active {
                        println!("Left Clicked x={}, y={}", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Reveal)
                            .expect("Failed to play square");
                    } else if resp.clicked_by(egui::PointerButton::Secondary) && active {
                        println!("Right Clicked x={}, y={}", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Flag)
                            .expect("Failed to play square");
                    } else if resp.clicked_by(egui::PointerButton::Middle) && active {
                        println!("Right Clicked x={}, y={}", x, y);
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

        if self.game_state == GameState::EndedLoss {
            egui::Image::new(egui::include_image!("../assets/loss.png")).paint_at(ui, rect);
        } else if self.game_state == GameState::EndedWin {
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
        if sqr.is_mine() && (sqr.is_revealed || self.game_state == GameState::EndedLoss) {
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

// fn main() -> Result<(), Error> {
//     let mut gb = GameBoard::new(10, 10);
//     gb.squares[1] = Square::default_mine();
//     gb.squares[10] = Square::default_mine();

//     gb.populate_numerals()?;

//     println!(" ");
//     gb.play(1, 1, RevealType::Reveal)?;
//     gb.print();
//     println!(
//         "Is Win: {}, Is Loss: {}",
//         gb.is_win_configuration(),
//         gb.is_loss_configuration()
//     );

//     gb.play(1, 1, RevealType::Chord)?;
//     gb.print();
//     println!(
//         "Is Win: {}, Is Loss: {}",
//         gb.is_win_configuration(),
//         gb.is_loss_configuration()
//     );

//     gb.play(0, 0, RevealType::Flag)?;
//     gb.play(2, 2, RevealType::Flag)?;
//     gb.print();
//     println!(
//         "Is Win: {}, Is Loss: {}",
//         gb.is_win_configuration(),
//         gb.is_loss_configuration()
//     );

//     gb.play(1, 1, RevealType::Chord)?;
//     gb.print();
//     println!(
//         "Is Win: {}, Is Loss: {}",
//         gb.is_win_configuration(),
//         gb.is_loss_configuration()
//     );

//     Ok(())
// }
