mod minesweeper;

use minesweeper::*;

fn main() -> Result<(), Error> {
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
