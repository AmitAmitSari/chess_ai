use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::time::Instant;
use text_io::read;

use crate::two_player_game::{Game, GameState, Player, Scored};
use crate::alpha_beta::{get_next_move, MoveResult, alpha_beta, A};
use crate::bit_help::{coord_to_index, Dir, index, index_to_place, place_to_coord, ray, ray_until_blocker};
use crate::chess_impl::{Chess, Move};
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use crate::utils::get_time;

mod two_player_game;
mod alpha_beta;
mod chess_impl;
mod bit_help;
mod move_generation;
mod tests;
mod utils;


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


fn play_game_chess_com(game: &mut Chess, player: Player) -> GameState {
    loop {
        if game.current_player() == player {
            let om = get_next_move(game, 8, i32::MAX);
            match om {
                MoveResult { chess_move: Some(m), .. } => {
                    println!("{}", m);
                    game.do_move(m);
                }

                MoveResult { chess_move: None, .. } => {
                    return GameState::PLAYER1WIN
                }
            }
        } else {
            let mut move_string: String = read!();

            while game.possible_moves().into_iter().filter(|m| m.to_string() == move_string).nth(0).is_none() {
                println!("ERROR");
                move_string = read!();
            }
            println!("GOOD");

            game.do_move(game.possible_moves().into_iter().filter(|m| m.to_string() == move_string).nth(0).unwrap());
        }
    }
}

fn output_game_state(game: &Chess) {
    let pieces = game.all_pieces();
    println!("{}", pieces.len());
    for (x, y, player, piece) in pieces {
        let info_to_char = [["P", "N", "B", "R", "Q", "K"], ["p", "n", "b", "r", "q", "k"]];
        println!("{} {} {}", x, y, info_to_char[player as usize][piece as usize]);
    }
}

fn output_move(m: Move) {
    println!("{}", m.serialize())
}

fn output_possible_moves(game: &Chess) {
    let moves = game.possible_moves();
    println!("{}", moves.len());
    for m in moves {
        output_move(m);
    }
}

fn input_move(game: &Chess) -> (Option<Move>, i32) {
    let move_string: String = read!("{}__");
    let timeout_seconds: i32 = read!("{}\n");
    let _m: Option<Move> = game.possible_moves().into_iter().filter(|m| m.serialize() == move_string).nth(0);
    if _m.is_some() && timeout_seconds > 0 {
        println!("GOOD");
    } else {
        eprintln!("ERROR");
        println!("ERROR");
    }

    let max_timestamp_ms = get_time() + timeout_seconds * 1000;
    return (_m, max_timestamp_ms);
}

fn play_game_my_front(human_as: Player, depth: i32) {
    /*
        game state
        if human_turn: possible_moves
            if no moves: "AI WIN"
        else:

     */
    let mut game = Chess::new();
    let mut max_timestamp_ms = 0;
    let mut om;

    loop {
        output_game_state(&game);

        if game.current_player() == human_as {
            output_possible_moves(&game);
            if game.possible_moves().len() == 0 {
                println!("AI WIN");
                break;
            }

            let tup = input_move(&game);
            om = tup.0; max_timestamp_ms = tup.1;
            while om.is_none() {
                let tup = input_move(&game);
                om = tup.0; max_timestamp_ms = tup.1;
            }
            game.do_move(om.unwrap());
        } else {
            let om = get_next_move(&mut game, depth, max_timestamp_ms);
            match om {
                MoveResult { chess_move: Some(m), ..} => {
                    game.do_move(m);
                    eprintln!("Move from depth: {}", om.move_from_depth);
                }

                MoveResult { chess_move: None, .. } => {
                    println!("HUMAN WIN");
                    break;
                }
            }
        }
    }
}

fn play_self() {
    let mut chess = Chess::new();
    let mut turns = 0;
    let start = Instant::now();
    loop {
        println!("At move: {}, took {:?}", turns, start.elapsed());
        let m = get_next_move(&mut chess, 8, i32::MAX);
        match m {
            MoveResult { chess_move: None, .. } => { break; }
            MoveResult { chess_move: Some(m_), .. } => { println!("Found move: {}", m_); chess.do_move(m_); }
        }
        turns += 1;
    }
}

fn print_state_at(fen: &str, move_str: &str, depth: i32) {
    let mut chess = Chess::new();
    chess.setup_fen_string("8/1p6/1k6/4r1NP/1P6/2P2R2/1b3PP1/5K2 w - - 0 1");
    let a = Chess::MIN_INFINITY;
    let b = Chess::MAX_INFINITY;

    println!("Alpha beta score: {}", alpha_beta(&mut chess, depth, a, b, 0, &mut HashMap::with_hasher(A {}), &mut 0, i32::MAX).unwrap() );
    chess.do_move(chess.possible_moves().into_iter().filter(|m| m.to_string() == move_str).nth(0).unwrap());

    for d in (0..depth).rev() {
        // println!("Alpha beta score: {}", alpha_beta(&mut chess, d, a, b, 0, &mut HashMap::new()) );
        let m = get_next_move(&mut chess, d, i32::MAX).chess_move.unwrap();
        println!("Do Move: {}", m);
        chess.do_move(m);
    }

    chess.console_draw();
}


fn main() {
    play_game_my_front(PLAYER1, 7);
}
