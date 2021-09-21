use crate::two_player_game::Player::PLAYER1;
use std::fmt::{Debug, Display};

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Player { PLAYER1 = 0, PLAYER2 = 1 }

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum GameState { PLAYER1WIN, PLAYER2WIN, TIE, PLAYING}

impl GameState {
    pub fn winning_player(&self) -> Option<Player>{
        match *self {
            GameState::PLAYER1WIN => Some(Player::PLAYER1),
            GameState::PLAYER2WIN => Some(Player::PLAYER2),
            _ => None
        }
    }
}

pub trait Game {
    type T;

    fn current_player(&self) -> Player;

    // Return a vector of possible moves.
    fn possible_moves(&self) -> Vec<Self::T>;

    // Mutate the board state doing the move 'play'
    fn do_move(&mut self, play: &Self::T);

    // Undo the last move.
    fn undo_move(&mut self);

    // Return
    fn game_state(&self) -> GameState;

    fn console_draw(&self) {}
}

pub trait Scored {
    type ScoreType: Ord + Clone + Eq + PartialEq + Copy + Debug + Display;

    const MAX_INFINITY: Self::ScoreType;
    const MIN_INFINITY: Self::ScoreType;
    const MAX_SCORE: Self::ScoreType;
    const NEUTRAL_SCORE: Self::ScoreType;
    const MIN_SCORE: Self::ScoreType;

    // player 1 score is positive player 2 score is negative.
    fn get_score(&self) -> Self::ScoreType;
}
