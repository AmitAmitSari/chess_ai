use std::fmt::{Display, Formatter};
use crate::two_player_game::{Game};
use crate::alpha_beta::get_next_move;
use crate::bit_help::{coord_to_index, Dir, index, index_to_place, place_to_coord, ray, ray_until_blocker};
use crate::chess_impl::{Chess, Move};
use crate::two_player_game::GameState::PLAYING;

mod two_player_game;
mod alpha_beta;
mod xo_impl;
mod chess_impl;
mod bit_help;
mod move_generation;


fn count_positions(chess: &mut Chess, depth: i32) -> usize {
    if depth == 1 {
        return chess.possible_moves().len();
    }

    let moves = chess.possible_moves();
    let mut res = 0;
    for m in moves {
        chess.do_move(m);
        res += count_positions(chess, depth - 1);
        chess.undo_move();
    }
    res
}

fn place_to_letters(place: u64) -> String {
    let (x, y) = place_to_coord(place);
    let lett: String = "hgfedcba".chars().map(|c| c.to_string()).nth(x as usize).unwrap();
    let num: String = "12345678".chars().map(|c| c.to_string()).nth(y as usize).unwrap();
    return lett + &num;
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&(place_to_letters(self.from) + &place_to_letters(self.to)))
    }
}

fn print_u64(map: u64) {
    for y in 0..8 {
        for x in 0..8 {
            if map & index_to_place(coord_to_index((x, y))) != 0 {
                print!("1")
            } else {
                print!("0")
            }
        }
        println!()
    }
}

fn main() {
    let mut chess = Chess::new();

    for &move_str in ["d2d3", "e7e5", "e1d2", "e5e4"].iter() {
        chess.do_move(chess.possible_moves().into_iter().filter(|m| m.to_string() == move_str).nth(0).unwrap());
    }

    chess.console_draw();
    let depth = 1;

    for i in 1..depth+1 {
        println!("{}", count_positions(&mut chess, i));
    }

    for m in chess.possible_moves() {
        chess.do_move(m);
        let cnt = if depth > 1 {
            count_positions(&mut chess, depth - 1)
        } else { 0 };
        let x = chess.undo_move();
        println!("{}, {}", cnt, x);
    }
}
