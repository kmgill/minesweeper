use anyhow::Result;
use itertools::iproduct;
use rand::prelude::*;

/// Indicates some sort of error related to initialization and play on the gameboard
#[derive(Debug)]
#[allow(dead_code)]
pub enum Error {
    ExcessiveMines,
    InvalidCoordinates,
    IndexOutOfBounds,
    InvalidCascade,
    UnexpectedResult,
}

/// Represents the type of a square as to the presence of a mine
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SquareType {
    Empty,
    Mine,
}

/// Representation of a single minesweeper square.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Square {
    pub is_revealed: bool,
    pub is_flagged: bool,
    pub square_type: SquareType,
    pub numeral: u32,
}

impl Default for Square {
    fn default() -> Self {
        Square {
            is_revealed: false,
            is_flagged: false,
            numeral: 0,
            square_type: SquareType::Empty,
        }
    }
}

impl Square {
    pub fn default_mine() -> Self {
        Square {
            is_revealed: false,
            is_flagged: false,
            numeral: 0,
            square_type: SquareType::Mine,
        }
    }

    pub fn is_mine(&self) -> bool {
        self.square_type == SquareType::Mine
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        if self.is_flagged {
            print!(" > ");
        } else if !self.is_revealed {
            print!(" - ");
        } else if self.is_mine() {
            print!(" X ");
        } else if self.numeral > 0 {
            print!(" {} ", self.numeral)
        } else {
            print!("   ");
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

impl From<(u32, u32)> for Coordinate {
    fn from(xy: (u32, u32)) -> Self {
        Coordinate { x: xy.0, y: xy.1 }
    }
}

impl Coordinate {
    #[allow(dead_code)]
    pub fn matches(&self, x: u32, y: u32) -> bool {
        self.x == x && self.y == y
    }
}

pub enum RevealType {
    Reveal,
    Chord,
    Flag,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PlayResult {
    Flagged(bool),
    Explosion(Coordinate), // Loss
    NoChange,
    Revealed(Coordinate),
    CascadedReveal(Vec<PlayResult>),
}

#[derive(Debug, Clone)]
/// Representation of a minesweeper game board
pub struct GameBoard {
    pub width: u32,
    pub height: u32,
    pub num_mines: u32,
    pub squares: Vec<Square>,
    pub is_populated: bool,
}

impl GameBoard {
    pub fn new(width: u32, height: u32) -> Self {
        GameBoard {
            width,
            height,
            num_mines: 0,
            squares: (0..width * height).map(|_| Square::default()).collect(),
            is_populated: false,
        }
    }

    #[allow(dead_code)]
    pub fn new_populated(width: u32, height: u32, num_mines: u32) -> Result<GameBoard, Error> {
        let mut gb = Self::new(width, height);
        gb.populate_mines(num_mines)?;
        gb.populate_numerals()?;
        Ok(gb)
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.squares = (0..self.width * self.height)
            .map(|_| Square::default())
            .collect();
    }

    #[allow(dead_code)]
    pub fn new_populated_around(
        width: u32,
        height: u32,
        num_mines: u32,
        keep_clear: Coordinate,
    ) -> Result<GameBoard, Error> {
        let mut gb = Self::new(width, height);
        gb.populate_mines_around(num_mines, Some(keep_clear))?;
        gb.populate_numerals()?;
        Ok(gb)
    }

    /// Convert x, y coordinate to vector index
    fn xy_to_idx(&self, x: u32, y: u32) -> u32 {
        y * self.width + x
    }

    fn idx_to_xy(&self, idx: u32) -> Result<Coordinate, Error> {
        if idx as usize > self.squares.len() - 1 {
            return Err(Error::IndexOutOfBounds);
        }

        Ok(Coordinate {
            x: idx % self.width,
            y: idx / self.width,
        })
    }

    fn get_square_by_idx(&self, idx: u32) -> Result<Square, Error> {
        if idx as usize >= self.squares.len() {
            Err(Error::InvalidCoordinates)
        } else {
            Ok(self.squares[idx as usize])
        }
    }

    pub fn get_square(&self, x: u32, y: u32) -> Result<Square, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else {
            self.get_square_by_idx(self.xy_to_idx(x, y))
        }
    }

    /// Determines whether a square contains a mine, allowing for negative
    /// and invalid coordinates.
    fn is_mine_protected(&self, x: i32, y: i32) -> bool {
        if x < 0 {
            return false;
        }
        if y < 0 {
            return false;
        }

        match self.get_square(x as u32, y as u32) {
            Ok(sqr) => sqr.is_mine(),
            _ => false,
        }
    }

    fn is_flagged_protected(&self, x: i32, y: i32) -> bool {
        if x < 0 {
            return false;
        }
        if y < 0 {
            return false;
        }

        match self.get_square(x as u32, y as u32) {
            Ok(sqr) => sqr.is_flagged,
            _ => false,
        }
    }

    fn flagged_neighbor_count(&self, x: u32, y: u32) -> Result<u32, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else {
            Ok(iproduct!(-1_i32..2_i32, -1_i32..2_i32)
                .map(|(dx, dy)| {
                    if self.is_flagged_protected(x as i32 + dx, y as i32 + dy) {
                        1
                    } else {
                        0
                    }
                })
                .collect::<Vec<u32>>()
                .into_iter()
                .sum())
        }
    }

    /// Determine how many mines a given square touches.
    fn mined_neighbor_count(&self, x: u32, y: u32) -> Result<u32, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else {
            Ok(iproduct!(-1_i32..2_i32, -1_i32..2_i32)
                .map(|(dx, dy)| {
                    if self.is_mine_protected(x as i32 + dx, y as i32 + dy) {
                        1
                    } else {
                        0
                    }
                })
                .collect::<Vec<u32>>()
                .into_iter()
                .sum())
        }
    }

    pub fn populate_mines_around(
        &mut self,
        num_mines: u32,
        keep_clear: Option<Coordinate>,
    ) -> Result<(), Error> {
        if num_mines > self.width * self.height {
            Err(Error::ExcessiveMines)
        } else {
            self.num_mines = num_mines;

            let mut mines_placed = 0;
            while mines_placed < num_mines {
                let random_idx = rand::thread_rng().gen_range(0..self.squares.len() - 1);

                if let Some(kc) = &keep_clear {
                    if !self.get_square_by_idx(random_idx as u32)?.is_mine()
                        && *kc != self.idx_to_xy(random_idx as u32)?
                    {
                        self.squares[random_idx] = Square::default_mine();
                        mines_placed += 1;
                    }
                } else if !self.get_square_by_idx(random_idx as u32)?.is_mine() {
                    self.squares[random_idx] = Square::default_mine();
                    mines_placed += 1;
                }
            }
            self.is_populated = true;
            Ok(())
        }
    }

    pub fn populate_mines(&mut self, num_mines: u32) -> Result<(), Error> {
        self.populate_mines_around(num_mines, None)
    }

    pub fn populate_numerals(&mut self) -> Result<(), Error> {
        iproduct!(0..self.width, 0..self.height).for_each(|(x, y)| {
            let idx = self.xy_to_idx(x, y);
            self.squares[idx as usize].numeral = self.mined_neighbor_count(x, y).unwrap_or(0);
        });

        Ok(())
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.squares[self.xy_to_idx(x, y) as usize].print();
            }
            println!();
        }
    }

    /// Toggles the flagged state of a square.
    /// Returns the updated flagged state of the square.
    ///
    /// A revealed square cannot be flagged
    ///
    pub fn flag(&mut self, x: u32, y: u32) -> Result<PlayResult, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else {
            let idx = self.xy_to_idx(x, y);
            let sqr = self.get_square_by_idx(idx)?;
            if !sqr.is_revealed {
                self.squares[idx as usize].is_flagged = !sqr.is_flagged;
                Ok(PlayResult::Flagged(self.squares[idx as usize].is_flagged))
            } else {
                Ok(PlayResult::NoChange) // Maybe return false instead?
            }
        }
    }

    pub fn cascade_from(&mut self, x: u32, y: u32) -> Result<PlayResult, Error> {
        if x >= self.width || y >= self.height {
            return Err(Error::InvalidCoordinates);
        }

        let idx = self.xy_to_idx(x, y);

        if self.squares[idx as usize].is_mine()
            || self.squares[idx as usize].is_flagged
            || self.squares[idx as usize].numeral > 0
        {
            return Err(Error::InvalidCascade);
        }
        self.squares[idx as usize].is_revealed = true;

        // TODO: Probably not
        let results = iproduct!(-1_i32..2_i32, -1_i32..2_i32)
            .map(|(dx, dy)| self.reveal_protected(x as i32 + dx, y as i32 + dy))
            .collect::<Vec<PlayResult>>();

        Ok(PlayResult::CascadedReveal(results))
    }

    // Defines a single square reveal
    pub fn reveal(&mut self, x: u32, y: u32) -> Result<PlayResult, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else {
            let idx = self.xy_to_idx(x, y);
            let sqr = self.get_square_by_idx(idx)?;

            if sqr.is_mine() && !sqr.is_flagged {
                // If the square is a mine and it's not flagged (unprotected)
                self.squares[idx as usize].is_revealed = true;
                Ok(PlayResult::Explosion(Coordinate::from((x, y))))
            } else if !sqr.is_mine() && !sqr.is_flagged && !sqr.is_revealed {
                // if the square is not a mine, is unflagged, and is unrevealed
                if self.squares[idx as usize].numeral == 0 {
                    // If it's a non-numeral square, we can auto-chord it
                    self.cascade_from(x, y)
                } else {
                    // Otherwise, reveal the single square, and set it as so
                    self.squares[idx as usize].is_revealed = true;
                    Ok(PlayResult::Revealed(Coordinate::from((x, y))))
                }
            } else {
                // Otherwise no change (user tried to reveal an already revealed square)
                Ok(PlayResult::NoChange)
            }
        }
    }

    fn reveal_protected(&mut self, x: i32, y: i32) -> PlayResult {
        if x < 0 {
            return PlayResult::NoChange;
        }
        if y < 0 {
            return PlayResult::NoChange;
        }

        match self.reveal(x as u32, y as u32) {
            Ok(res) => res,
            Err(_) => PlayResult::NoChange,
        }
    }

    /// Determine whether a given square can be chorded.
    ///
    /// Has a zero numeral: yes
    /// Has same number of neighbors flagged as numeral: yes
    /// Does *not* determine if the square can be *safely* chorded
    /// If the number of flagged neighbors is greated than the numeral, then
    ///     there is an abiguity and the square cannot be chorded.
    pub fn can_chord_square(&self, x: u32, y: u32) -> Result<bool, Error> {
        if x >= self.width || y >= self.height {
            return Err(Error::InvalidCoordinates);
        }
        let sqr = self.get_square(x, y)?;

        // Is it a blank square or does the numeral match the number of flagged neighbors
        if sqr.numeral == 0 || sqr.numeral == self.flagged_neighbor_count(x, y)? {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Executes a 'chord' reveal on the requested square.
    pub fn chord(&mut self, x: u32, y: u32) -> Result<PlayResult, Error> {
        if x >= self.width || y >= self.height {
            Err(Error::InvalidCoordinates)
        } else if !self.can_chord_square(x, y)? {
            Ok(PlayResult::NoChange)
        } else {
            let results = iproduct!(-1_i32..2_i32, -1_i32..2_i32)
                .map(|(dx, dy)| self.reveal_protected(x as i32 + dx, y as i32 + dy))
                .collect::<Vec<PlayResult>>();

            Ok(PlayResult::CascadedReveal(results))
        }
    }

    /// Determine if the board is in a winning configuration.
    ///
    /// Conditions
    /// - All non-mine squares are revealed (mined need not be flagged)
    #[allow(dead_code)]
    pub fn is_win_configuration(&self) -> bool {
        self.squares
            .clone()
            .into_iter()
            .map(|s| if !s.is_mine() && !s.is_revealed { 1 } else { 0 })
            .collect::<Vec<u32>>()
            .into_iter()
            .sum::<u32>()
            == 0_u32
    }

    #[allow(dead_code)]
    pub fn is_loss_configuration(&self) -> bool {
        self.squares
            .clone()
            .into_iter()
            .map(|s| if s.is_mine() && s.is_revealed { 1 } else { 0 })
            .collect::<Vec<u32>>()
            .into_iter()
            .sum::<u32>()
            > 0_u32
    }

    pub fn play(&mut self, x: u32, y: u32, reveal_type: RevealType) -> Result<PlayResult, Error> {
        match reveal_type {
            RevealType::Flag => self.flag(x, y),
            RevealType::Reveal => self.reveal(x, y),
            RevealType::Chord => self.chord(x, y),
        }
    }

    pub fn num_flags(&self) -> u32 {
        self.squares
            .clone()
            .into_iter()
            .map(|s| if s.is_flagged { 1 } else { 0 })
            .collect::<Vec<u32>>()
            .into_iter()
            .sum::<u32>()
    }

    // Don't cheat
    #[allow(dead_code)]
    pub fn flag_all_mines(&mut self) {
        for sqr in self.squares.iter_mut() {
            sqr.is_flagged = sqr.is_mine();
        }
    }

    #[allow(dead_code)]
    pub fn reset_existing(&mut self) {
        for sqr in self.squares.iter_mut() {
            sqr.is_flagged = false;
            sqr.is_revealed = false;
        }
    }
}

#[test]
fn test_squares() {
    let sq1 = Square::default();
    assert!(!sq1.is_mine());
    assert!(!sq1.is_revealed);
    assert!(!sq1.is_flagged);

    let sq1 = Square::default_mine();
    assert!(sq1.is_mine());
    assert!(!sq1.is_revealed);
    assert!(!sq1.is_flagged);
}

#[test]
fn test_gameboard_new() {
    let gb = GameBoard::new(10, 10);
    assert_eq!(gb.height, 10);
    assert_eq!(gb.width, 10);
    assert_eq!(gb.num_mines, 0);
    assert_eq!(gb.squares.len(), 100);
}

#[test]
fn test_idx_to_xy() -> Result<(), Error> {
    let gb = GameBoard::new(3, 3);

    assert!(gb.idx_to_xy(0)?.matches(0, 0));
    assert!(gb.idx_to_xy(1)?.matches(1, 0));
    assert!(gb.idx_to_xy(2)?.matches(2, 0));
    assert!(gb.idx_to_xy(3)?.matches(0, 1));
    assert!(gb.idx_to_xy(4)?.matches(1, 1));
    assert!(gb.idx_to_xy(5)?.matches(2, 1));
    assert!(gb.idx_to_xy(6)?.matches(0, 2));
    assert!(gb.idx_to_xy(7)?.matches(1, 2));
    assert!(gb.idx_to_xy(8)?.matches(2, 2));

    Ok(())
}

#[test]
fn test_populate_num_mines() -> Result<(), Error> {
    let num_mines = 20;
    let gb = GameBoard::new_populated(100, 100, num_mines)?;
    assert_eq!(gb.num_mines, num_mines);
    let mut num_mines_found = 0;

    for x in 0..100 {
        for y in 0..100 {
            if gb.get_square(x, y)?.is_mine() {
                num_mines_found += 1;
            }
        }
    }
    assert_eq!(num_mines, num_mines_found);
    Ok(())
}

#[test]
fn test_excessive_mines() {
    let mut gb = GameBoard::new(2, 2);
    match gb.populate_mines(5) {
        Err(Error::ExcessiveMines) => {}
        _ => panic!("Invalid response"),
    }
}

#[test]
fn test_invalid_coordinates() {
    let gb = GameBoard::new(2, 2);

    match gb.get_square(1, 1) {
        Ok(_) => {}
        _ => panic!("Invalid response"),
    };

    match gb.get_square(3, 3) {
        Err(Error::InvalidCoordinates) => {}
        _ => panic!("Invalid response"),
    };

    match gb.get_square(1, 3) {
        Err(Error::InvalidCoordinates) => {}
        _ => panic!("Invalid response"),
    };

    match gb.get_square(3, 1) {
        Err(Error::InvalidCoordinates) => {}
        _ => panic!("Invalid response"),
    };
}

#[test]
fn test_mined_neighbor_count() -> Result<(), Error> {
    let mut gb = GameBoard::new(3, 3);
    gb.squares[0] = Square::default_mine();
    assert_eq!(gb.mined_neighbor_count(1, 0)?, 1);
    assert_eq!(gb.mined_neighbor_count(1, 1)?, 1);
    assert_eq!(gb.mined_neighbor_count(0, 1)?, 1);
    assert_eq!(gb.mined_neighbor_count(2, 0)?, 0);
    assert_eq!(gb.mined_neighbor_count(2, 1)?, 0);
    assert_eq!(gb.mined_neighbor_count(2, 2)?, 0);
    assert_eq!(gb.mined_neighbor_count(0, 2)?, 0);
    assert_eq!(gb.mined_neighbor_count(1, 2)?, 0);

    gb.squares[8] = Square::default_mine();
    assert_eq!(gb.mined_neighbor_count(1, 1)?, 2);
    assert_eq!(gb.mined_neighbor_count(1, 2)?, 1);
    assert_eq!(gb.mined_neighbor_count(2, 1)?, 1);
    assert_eq!(gb.mined_neighbor_count(2, 0)?, 0);
    assert_eq!(gb.mined_neighbor_count(0, 2)?, 0);

    Ok(())
}

#[test]
fn test_flagged_neighbor_count() -> Result<(), Error> {
    let mut gb = GameBoard::new(3, 3);

    gb.play(0, 0, RevealType::Flag)?;
    gb.play(2, 2, RevealType::Flag)?;
    assert_eq!(gb.flagged_neighbor_count(1, 1)?, 2);

    gb.play(2, 2, RevealType::Flag)?;
    assert_eq!(gb.flagged_neighbor_count(1, 1)?, 1);

    gb.play(0, 0, RevealType::Flag)?;
    gb.play(0, 1, RevealType::Flag)?;
    gb.play(1, 0, RevealType::Flag)?;
    assert_eq!(gb.flagged_neighbor_count(0, 0)?, 2);

    Ok(())
}

#[test]
fn test_chord() -> Result<(), Error> {
    let mut gb = GameBoard::new(10, 10);
    gb.squares[1] = Square::default_mine();
    gb.squares[10] = Square::default_mine();
    gb.populate_numerals()?;

    gb.flag(1, 0)?;
    gb.flag(0, 1)?;

    let results = gb.chord(0, 0)?;

    match results {
        PlayResult::CascadedReveal(results_vec) => {
            assert_eq!(results_vec.len(), 9);
            match &results_vec[0] {
                PlayResult::NoChange => {}
                _ => panic!("Result of -1,-1 should have been NoChange"),
            };
            match &results_vec[4] {
                PlayResult::Revealed(c) => {
                    assert_eq!(c.x, 0);
                    assert_eq!(c.y, 0);
                }
                _ => panic!("Result of 0,0 should have been a reveal"),
            };
            match &results_vec[5] {
                PlayResult::NoChange => {}
                _ => panic!("Result of 0,0 should have been a NoCHange (flagged mine)"),
            };
            match &results_vec[7] {
                PlayResult::NoChange => {}
                _ => panic!("Result of 0,0 should have been a NoCHange (flagged mine)"),
            };
            match &results_vec[8] {
                PlayResult::Revealed(c) => {
                    assert_eq!(c.x, 1);
                    assert_eq!(c.y, 1);
                }
                _ => panic!("Result of 1,1 should have been a reveal"),
            };
        }
        _ => panic!("Invalid chord response"),
    };

    Ok(())
}

#[test]
fn test_simple_plays() -> Result<(), Error> {
    let mut gb = GameBoard::new(3, 3);

    // Mine is in 0, 0
    gb.squares[0] = Square::default_mine();

    // Flag the mine, should return true Flagged
    match gb.play(0, 0, RevealType::Flag) {
        Ok(PlayResult::Flagged(s)) => assert!(s),
        _ => panic!("Should have returned true in PlayResult::Flagged"),
    };

    // Attempt to reveal the flagged mine, should result in no change
    match gb.play(0, 0, RevealType::Reveal) {
        Ok(PlayResult::NoChange) => {}
        _ => panic!("Revealing on a flagged square should result in RevealType::Reveal"),
    };

    // Unflag the mine at 0,0. Should result in a false Flagged
    match gb.play(0, 0, RevealType::Flag) {
        Ok(PlayResult::Flagged(s)) => assert!(!s),
        _ => panic!("Should have returned false in PlayResult::Flagged"),
    };

    // Try to reveal on 0,0. Should explode
    match gb.play(0, 0, RevealType::Reveal) {
        Ok(PlayResult::Explosion(c)) => {
            assert_eq!(c.x, 0);
            assert_eq!(c.y, 0);
        }
        _ => panic!("Should have exploded"),
    };

    Ok(())
}

#[test]
fn test_simple_game_1() -> Result<(), Error> {
    let mut gb = GameBoard::new(10, 10);
    gb.squares[1] = Square::default_mine();
    gb.squares[10] = Square::default_mine();
    gb.squares[35] = Square::default_mine();

    gb.populate_numerals()?;

    println!(" ");
    gb.play(1, 1, RevealType::Reveal)?;
    gb.print();
    assert!(!gb.is_win_configuration());

    gb.play(2, 2, RevealType::Reveal)?;
    gb.print();
    assert!(!gb.is_win_configuration());

    gb.play(1, 0, RevealType::Flag)?;
    gb.print();
    assert!(!gb.is_win_configuration());

    gb.play(0, 1, RevealType::Flag)?;
    gb.print();
    assert!(!gb.is_win_configuration());

    gb.play(5, 3, RevealType::Flag)?;
    gb.print();
    assert!(!gb.is_win_configuration());

    gb.play(0, 0, RevealType::Chord)?;
    gb.print();
    assert!(gb.is_win_configuration());

    Ok(())
}

#[test]
fn test_simple_game_2() -> Result<(), Error> {
    // Constructs a simple game board by placing mines by the top left corner.
    // Chord to no effect. Flag the incorrect locations. Chord for a loss
    let mut gb = GameBoard::new(10, 10);
    gb.squares[1] = Square::default_mine();
    gb.squares[10] = Square::default_mine();

    gb.populate_numerals()?;

    println!(" ");
    gb.play(1, 1, RevealType::Reveal)?;
    gb.print();
    println!(
        "Is Win: {}, Is Loss: {}",
        gb.is_win_configuration(),
        gb.is_loss_configuration()
    );

    gb.play(1, 1, RevealType::Chord)?;
    gb.print();
    println!(
        "Is Win: {}, Is Loss: {}",
        gb.is_win_configuration(),
        gb.is_loss_configuration()
    );

    gb.play(0, 0, RevealType::Flag)?;
    gb.play(2, 2, RevealType::Flag)?;
    gb.print();
    println!(
        "Is Win: {}, Is Loss: {}",
        gb.is_win_configuration(),
        gb.is_loss_configuration()
    );

    gb.play(1, 1, RevealType::Chord)?;
    gb.print();
    println!(
        "Is Win: {}, Is Loss: {}",
        gb.is_win_configuration(),
        gb.is_loss_configuration()
    );

    Ok(())
}
