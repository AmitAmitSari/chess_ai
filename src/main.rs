use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Instant;
use text_io::read;

use crate::two_player_game::{Game, GameState, Player, Scored};
use crate::alpha_beta::{get_next_move, alpha_beta};
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
            let om = get_next_move(game, 7);
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


fn play_self() {
    let mut chess = Chess::new();
    let mut turns = 0;
    let start = Instant::now();
    loop {
        println!("At move: {}, took {:?}", turns, start.elapsed());
        let m = get_next_move(&mut chess, 7);
        match m {
            None => { break; }
            Some(m_) => { println!("Found move: {}", m_); chess.do_move(m_); }
        }
        turns += 1;
    }
}

fn print_state_at(fen: &str, move_str: &str, depth: i32) {
    let mut chess = Chess::new();
    chess.setup_fen_string("8/1p6/1k6/4r1NP/1P6/2P2R2/1b3PP1/5K2 w - - 0 1");
    let a = Chess::MIN_INFINITY;
    let b = Chess::MAX_INFINITY;

    println!("Alpha beta score: {}", alpha_beta(&mut chess, depth, a, b, 0, &mut HashMap::new(), &mut 0) );
    chess.do_move(chess.possible_moves().into_iter().filter(|m| m.to_string() == move_str).nth(0).unwrap());

    for d in (0..depth).rev() {
        // println!("Alpha beta score: {}", alpha_beta(&mut chess, d, a, b, 0, &mut HashMap::new()) );
        let m = get_next_move(&mut chess, d).unwrap();
        println!("Do Move: {}", m);
        chess.do_move(m);
    }

    chess.console_draw();
}


fn main() {
    play_self();
}
