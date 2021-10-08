use crate::two_player_game::{Game};
use crate::alpha_beta::get_next_move;
use crate::chess_impl::Chess;
use crate::two_player_game::GameState::PLAYING;

mod two_player_game;
mod alpha_beta;
mod xo_impl;
mod chess_impl;
mod bit_help;
mod move_generation;


fn main() {
    let mut chess = Chess::new();
    chess.console_draw();
}
