use std::collections::HashMap;

use crate::two_player_game::{Game, Player, Scored};
use crate::two_player_game::GameState;
use crate::two_player_game::GameState::{PLAYER1WIN, PLAYER2WIN};


#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
enum Square { X, O, E}

#[derive(Debug)]
pub struct Xo {
    history: Vec<usize>,
    state: [Square; 9],
    cur_player: Square
}


impl Xo {
    fn flip_player(&mut self) {
        self.cur_player = if self.cur_player == Square::X { Square::O } else { Square::X};
    }

    fn line_filled(&self, start: usize, step: usize) -> bool {
        return self.state[start] != Square::E && self.state[start..].iter().step_by(step).take(3).all(|&x| x == self.state[start])
    }

    pub fn new_game() -> Xo {
        Xo {
            history: vec![],
            state: [Square::E; 9],
            cur_player: Square::X
        }
    }
}

impl Game for Xo {
    type MoveType = usize;

    fn new() -> Self {
        Xo::new_game()
    }

    fn setup_new_game(&mut self) {
        self.history.clear();
        self.state.fill(Square::E);
        self.cur_player = Square::X;
    }

    fn current_player(&self) -> Player {
        if self.cur_player == Square::X { Player::PLAYER1 } else { Player::PLAYER2 }
    }

    fn possible_moves(&self) -> Vec<usize> {
        let mut possible: Vec<usize> = vec![];
        possible.reserve(9);
        for i in 0..9 {
            if self.state[i] == Square::E {
                possible.push(i);
            }
        }
        possible
    }

    fn do_move(&mut self, play: usize) {
        self.state[play] = self.cur_player;
        self.flip_player();
        self.history.push(play);
    }

    fn undo_move(&mut self) -> usize {
        let last = self.history.pop().unwrap();
        self.state[last] = Square::E;
        self.flip_player();
        last
    }

    fn game_state(&self) -> GameState {
        let xo_to_12: HashMap<Square, GameState> = [
            (Square::X, PLAYER1WIN),
            (Square::O, PLAYER2WIN)
        ]
            .iter().cloned().collect();

        for i in 0..3 {
            // Columns
            if self.line_filled(i, 3) {
                return xo_to_12[&self.state[i]];
            }
            // Rows
            if self.line_filled(i*3, 1) {
                return xo_to_12[&self.state[i*3]];
            }
        }

        // Main diagonal
        if self.line_filled(0, 4) {
            return xo_to_12[&self.state[0]];
        }
        // Other diagonal
        if self.line_filled(2, 2) {
            return xo_to_12[&self.state[2]];
        }

        if self.state.iter().all(|&x| x != Square::E) {
            return GameState::TIE;
        }

        GameState::PLAYING
    }

    fn console_draw(&self) {
        for i in 0..3 {
            println!("{:?}", &self.state[i*3..i*3+3])
        }
    }
}

impl Scored for Xo {
    type ScoreType = i32;
    const MAX_INFINITY: Self::ScoreType = 100;
    const MIN_INFINITY: Self::ScoreType = -100;

    const MAX_SCORE: Self::ScoreType = 1;
    const NEUTRAL_SCORE: Self::ScoreType = 0;
    const MIN_SCORE: Self::ScoreType = -1;

    fn get_score(&self) -> Self::ScoreType {
        match self.game_state() {
            PLAYER1WIN => Self::MAX_SCORE,
            PLAYER2WIN => Self::MIN_SCORE,
            GameState::TIE => Self::NEUTRAL_SCORE,
            GameState::PLAYING => Self::NEUTRAL_SCORE
        }
    }
}
