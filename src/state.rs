use crate::constants::*;
use crate::enums::*;
use crate::Coordinate;
use std::time::Instant;

#[derive(Clone)]
pub struct GameSettings {
    pub width: u32,
    pub height: u32,
    pub num_mines: u32,
    pub use_numerals: bool,
    pub ui_width: f32,
    pub ui_height: f32,
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

#[derive(Clone)]
pub struct AppState {
    pub game_state: GameState,
    pub game_started: Instant,
    pub game_finished: Instant,
    pub game_settings: GameSettings,
    pub difficulty: GameDifficulty,
    pub left_click_chord: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game_state: GameState::NotStarted,
            game_started: Instant::now(),
            game_finished: Instant::now(),
            game_settings: GameSettings::intermediate(),
            difficulty: GameDifficulty::Intermediate,
            left_click_chord: false,
        }
    }
}
