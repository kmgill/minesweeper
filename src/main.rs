use anyhow::Result;
use itertools::iproduct;
use rand::prelude::*;

/// Indicates some sort of error related to initialization and play on the gameboard
#[derive(Debug)]
enum Error {
    ExcessiveMines,
    InvalidCoordinates,
    AttemptToFlagRevealedSquare,
}

/// Represents the type of a square as to the presence of a mine
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SquareType {
    Empty,
    Mine,
}

/// Representation of a single minesweeper square.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Square {
    is_revealed: bool,
    is_flagged: bool,
    square_type: SquareType,
    numeral: u32,
}

impl Square {
    pub fn default() -> Self {
        Square {
            is_revealed: false,
            is_flagged: false,
            numeral: 0,
            square_type: SquareType::Empty,
        }
    }

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

    pub fn print(&self) {
        if self.is_mine() {
            print!(" X ");
        } else {
            if self.numeral == 0 {
                print!(" - ");
            } else {
                print!(" {} ", self.numeral);
            }
        }
    }
}

struct Coordinate {
    x: u32,
    y: u32,
}

impl Coordinate {
    pub fn from(x: u32, y: u32) -> Self {
        Coordinate { x, y }
    }
}

enum RevealType {
    Reveal,
    Chord,
    Flag,
}

enum PlayResult {
    Flagged(bool),
    Explosion(Coordinate),
    NoChange,
    Revealed(Coordinate),
    ChordReveal(Vec<PlayResult>),
}

/// Representation of a minesweeper game board
struct GameBoard {
    width: u32,
    height: u32,
    num_mines: u32,
    squares: Vec<Square>,
}

impl GameBoard {
    pub fn new(width: u32, height: u32) -> Self {
        GameBoard {
            width: width,
            height: height,
            num_mines: 0,
            squares: (0..width * height).map(|_| Square::default()).collect(),
        }
    }

    pub fn new_populated(width: u32, height: u32, num_mines: u32) -> Result<GameBoard, Error> {
        let mut gb = Self::new(width, height);
        gb.populate_mines(num_mines)?;
        gb.populate_numerals()?;
        Ok(gb)
    }

    fn xy_to_idx(&self, x: u32, y: u32) -> u32 {
        y * self.width + x
    }

    fn get_square_by_idx(&self, idx: u32) -> Result<Square, Error> {
        if idx as usize >= self.squares.len() {
            Err(Error::InvalidCoordinates)
        } else {
            Ok(self.squares[idx as usize].clone())
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

    /// Determine how many mines a given square touches.
    fn determine_numeral(&self, x: u32, y: u32) -> Result<u32, Error> {
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

    pub fn populate_mines(&mut self, num_mines: u32) -> Result<(), Error> {
        if num_mines > self.width * self.height {
            Err(Error::ExcessiveMines)
        } else {
            self.num_mines = num_mines;

            let mut mines_placed = 0;
            while mines_placed < num_mines {
                let random_idx = rand::thread_rng().gen_range(0..self.squares.len() - 1);
                if !self.get_square_by_idx(random_idx as u32)?.is_mine() {
                    self.squares[random_idx] = Square::default_mine();
                    mines_placed += 1;
                }
            }

            Ok(())
        }
    }

    pub fn populate_numerals(&mut self) -> Result<(), Error> {
        iproduct!(0..self.width, 0..self.height).for_each(|(x, y)| {
            let idx = self.xy_to_idx(x, y);
            self.squares[idx as usize].numeral = self.determine_numeral(x, y).unwrap_or(0);
        });

        Ok(())
    }

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
                Err(Error::AttemptToFlagRevealedSquare) // Maybe return false instead?
            }
        }
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
                Ok(PlayResult::Explosion(Coordinate::from(x, y)))
            } else if !sqr.is_mine() && !sqr.is_flagged && !sqr.is_revealed {
                // if the square is not a mine, is unflagged, and is unrevealed
                if self.squares[idx as usize].numeral == 0 {
                    // If it's a non-numeral square, we can auto-chord it
                    self.chord(x, y)
                } else {
                    // Otherwise, reveal the single square, and set it as so
                    self.squares[idx as usize].is_revealed = true;
                    Ok(PlayResult::Revealed(Coordinate::from(x, y)))
                }
            } else {
                // Otherwise no change (user tried to reveal an already revealed square)
                Ok(PlayResult::NoChange)
            }
        }
    }

    /// Executes a 'chord' reveal on the requested square.
    pub fn chord(&mut self, x: u32, y: u32) -> Result<PlayResult, Error> {
        if x >= self.width || y >= self.height {
            return Err(Error::InvalidCoordinates);
        } else {
        }

        unimplemented!()
    }

    pub fn play(&mut self, x: u32, y: u32, reveal_type: RevealType) -> Result<PlayResult, Error> {
        match reveal_type {
            RevealType::Flag => self.flag(x, y),
            RevealType::Reveal => self.reveal(x, y),
            _ => unimplemented!(),
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
fn test_determine_numeral() -> Result<(), Error> {
    let mut gb = GameBoard::new(3, 3);
    gb.squares[0] = Square::default_mine();
    assert_eq!(gb.determine_numeral(1, 0)?, 1);
    assert_eq!(gb.determine_numeral(1, 1)?, 1);
    assert_eq!(gb.determine_numeral(0, 1)?, 1);
    assert_eq!(gb.determine_numeral(2, 0)?, 0);
    assert_eq!(gb.determine_numeral(2, 1)?, 0);
    assert_eq!(gb.determine_numeral(2, 2)?, 0);
    assert_eq!(gb.determine_numeral(0, 2)?, 0);
    assert_eq!(gb.determine_numeral(1, 2)?, 0);

    gb.squares[8] = Square::default_mine();
    assert_eq!(gb.determine_numeral(1, 1)?, 2);
    assert_eq!(gb.determine_numeral(1, 2)?, 1);
    assert_eq!(gb.determine_numeral(2, 1)?, 1);
    assert_eq!(gb.determine_numeral(2, 0)?, 0);
    assert_eq!(gb.determine_numeral(0, 2)?, 0);

    Ok(())
}

fn main() -> Result<(), Error> {
    let gb = GameBoard::new_populated(30, 30, 100)?;
    gb.print();
    Ok(())
}
