mod minesweeper;
use std::time::Instant;

use minesweeper::*;

use anyhow::Result;

use eframe::egui;
use egui::Vec2;
use egui_extras::install_image_loaders;
use itertools::iproduct;

#[derive(Eq, PartialEq, Debug)]
enum GameState {
    NotStarted,
    Playing,
    Ended,
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_icon(load_icon())
            .with_inner_size(Vec2::new(417.0, 500.0))
            .with_resizable(false),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        ..Default::default()
    };

    let app = Box::new(MinesweeperFoo {
        gameboard: minesweeper::GameBoard::new_populated_around(
            10,
            10,
            10,
            Coordinate { x: 5, y: 0 },
        )
        .expect("Failed to generate a game board"),
        game_state: GameState::NotStarted,
        game_started: Instant::now(),
        game_finished: Instant::now(),
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

struct MinesweeperFoo {
    gameboard: minesweeper::GameBoard,
    game_state: GameState,
    game_started: Instant,
    game_finished: Instant,
}

impl eframe::App for MinesweeperFoo {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.on_update(ctx, frame).expect("Failed to update UI");
    }
}

impl MinesweeperFoo {
    fn start_game(&mut self) {
        self.game_state = GameState::NotStarted;
        self.game_started = std::time::Instant::now();
        self.gameboard.reset();
        self.gameboard.populate_mines(10);
        self.gameboard.populate_numerals();
    }

    fn on_update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Result<(), Error> {
        install_image_loaders(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if self.face_ui(ui).clicked() {
                    self.start_game();
                }

                self.game_board_ui(ui, self.game_state != GameState::Ended);

                ui.horizontal_centered(|ui| {
                    ui.label(format!(
                        "{} of {}",
                        self.gameboard.num_flags(),
                        self.gameboard.num_mines
                    ));

                    if self.game_state == GameState::Playing
                        && (self.gameboard.is_loss_configuration()
                            || self.gameboard.is_win_configuration())
                    {
                        self.game_state = GameState::Ended;
                        self.game_finished = Instant::now();
                    } else if self.game_state == GameState::Playing {
                        ui.label(format!(
                            "Time: {:.2}",
                            self.game_started.elapsed().as_secs_f64()
                        ));
                    } else if self.game_state == GameState::Ended {
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

    fn set_game_playing(&mut self) {
        if self.game_state == GameState::NotStarted {
            self.game_started = Instant::now();
            self.game_state = GameState::Playing;
        }
    }

    fn game_board_ui(&mut self, ui: &mut egui::Ui, active: bool) {
        egui::Grid::new("process_grid_outputs")
            .num_columns(10)
            .spacing([0.0, 0.0])
            .striped(true)
            .show(ui, |ui| {
                iproduct!(0..10, 0..10).for_each(|(y, x)| {
                    let sqr = self
                        .gameboard
                        .get_square(x, y)
                        .expect("Error retrieving square");

                    let resp = self.square_ui(ui, &sqr);
                    if resp.clicked_by(egui::PointerButton::Primary) && active {
                        println!("Left Clicked x={}, y={}", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Reveal)
                            .expect("Failed to play square");
                        self.set_game_playing();
                    } else if resp.clicked_by(egui::PointerButton::Secondary) && active {
                        println!("Right Clicked x={}, y={}", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Flag)
                            .expect("Failed to play square");
                        self.set_game_playing();
                    } else if resp.clicked_by(egui::PointerButton::Middle) && active {
                        println!("Right Clicked x={}, y={}", x, y);
                        self.gameboard
                            .play(x, y, RevealType::Chord)
                            .expect("Failed to play square");
                        self.set_game_playing();
                    }
                    if x == 9 {
                        ui.end_row();
                    }
                });
            });
    }

    fn face_ui(&self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(1.0, 1.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if self.gameboard.is_win_configuration() {
            egui::Image::new(egui::include_image!("../assets/win.png")).paint_at(ui, rect)
        } else if self.gameboard.is_loss_configuration() {
            egui::Image::new(egui::include_image!("../assets/loss.png")).paint_at(ui, rect)
        } else {
            egui::Image::new(egui::include_image!("../assets/happy.png")).paint_at(ui, rect)
        }

        response
    }

    fn square_ui(&self, ui: &mut egui::Ui, sqr: &minesweeper::Square) -> egui::Response {
        let desired_size = ui.spacing().interact_size.x * egui::vec2(1.0, 1.0);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        let visuals = ui.style().interact_selectable(&response, false);
        ui.painter()
            .rect(rect, 1.0, visuals.bg_fill, visuals.bg_stroke);

        egui::Image::new(egui::include_image!("../assets/blank.png")).paint_at(ui, rect);
        if sqr.is_mine() && sqr.is_revealed {
            egui::Image::new(egui::include_image!("../assets/mine.png")).paint_at(ui, rect);
        } else if sqr.is_flagged {
            egui::Image::new(egui::include_image!("../assets/unrevealed.png")).paint_at(ui, rect);
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
            egui::Image::new(egui::include_image!("../assets/unrevealed.png")).paint_at(ui, rect)
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
