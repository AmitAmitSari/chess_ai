use std::fmt::{Display, Formatter};
use std::time::Instant;
use text_io::read;

use crate::two_player_game::{Game, GameState, Player};
use crate::alpha_beta::get_next_move;
use crate::bit_help::{coord_to_index, Dir, index, index_to_place, place_to_coord, ray, ray_until_blocker};
use crate::chess_impl::{Chess, Move};
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;

mod two_player_game;
mod alpha_beta;
mod chess_impl;
mod bit_help;
mod move_generation;
mod tests;






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


fn play_game(game: &mut Chess, player: Player) -> GameState {
    loop {
        if game.current_player() == player {
            let om = get_next_move(game, 6);
            if om.is_none() {
                // todo: fix
                return GameState::PLAYER1WIN;
            } else {
                let m = om.unwrap();
                println!("{}", m);
                game.do_move(m);
            }
        } else {
            let mut move_string: String = read!();

            while game.possible_moves().into_iter().filter(|m| m.to_string() == move_string).nth(0).is_none() {
                println!("Move was unreadable, input another one.");
                move_string = read!();
            }

            game.do_move(game.possible_moves().into_iter().filter(|m| m.to_string() == move_string).nth(0).unwrap());
        }
    }
}


fn main() {
    let mut chess = Chess::new();
    let mut turns = 0;
    let start = Instant::now();
    loop {
        println!("At move: {}, took {:?}", turns, start.elapsed());
        let m = get_next_move(&mut chess, 6);
        match m {
            None => { break; }
            Some(m_) => { println!("Found move: {}", m_); chess.do_move(m_); }
        }
        turns += 1;
    }

    // for &move_str in [""; 0].iter(){
    //     chess.do_move(chess.possible_moves().into_iter().filter(|m| m.to_string() == move_str).nth(0).unwrap());
    // }
    //
    // chess.console_draw();
    // let depth = 6;
    //
    // for i in 1..depth+1 {
    //     println!("{}", count_positions(&mut chess, i));
    // }
    //
    // for m in chess.possible_moves() {
    //     chess.do_move(m);
    //     let cnt = if depth > 1 {
    //         count_positions(&mut chess, depth - 1)
    //     } else { 0 };
    //     let x = chess.undo_move();
    //     println!("{}, {}", cnt, x);
    // }
}
