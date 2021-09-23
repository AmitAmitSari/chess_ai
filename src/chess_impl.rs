use crate::two_player_game::{Game, Player, GameState};
use std::ops::{Index, IndexMut};
use crate::bit_help::{place, index, place_to_coord, coord_to_index};

#[derive(Copy, Clone)]
pub enum PieceType { PAWN = 0, KNIGHT = 1, BISHOP = 2, ROOK = 3, QUEEN = 4, KING = 5 }

// White is on top.
// The least significant bit is top left. going over the board rows first.
pub struct BoardState {
    // Array of two players, with an int per piece type.
    piece_state: [[u64; 6]; 2],

    // Lit bits haven't moved.
    castle_memory: u64,

    // The square the pawn passed over. -1 for not applicable
    en_passant_square: u64
}

impl BoardState {
    fn get_mut(&mut self, player: Player, piece_type: PieceType) -> &mut u64 {
        &mut self.piece_state[player as usize][piece_type as usize]
    }

    fn move_piece(&mut self, player: Player, piece_type: PieceType, from: u64, to: u64) {
        let mut piece_state = self.get_mut(player, piece_type);
        *piece_state &= !from;
        *piece_state |= to;
    }

}

pub struct Move {
    from: u64,
    to: u64,
    start_type: PieceType,
    end_type: PieceType,

    eaten_type: PieceType,
    // Set to 0 so that nothing is eaten.
    eaten_loc: u64,

    // The whole new castle memory.
    castle_memory: u64,
    // Is this move a castle?
    castle_flag: bool,
    // Is this move en passant?
    en_passant_flag: bool,

    moving_player: Player
}


pub struct Chess {
    current_player: Player,
    board: BoardState,
    history: Vec<Move>
}

impl Game for Chess {
    type T = Move;

    fn current_player(&self) -> Player {
        return self.current_player;
    }

    fn possible_moves(&self) -> Vec<Self::T> {
        todo!()
    }

    fn do_move(&mut self, play: Self::T) {

        *self.board.get_mut(play.moving_player, play.start_type) &= !play.from;
        *self.board.get_mut(play.moving_player, play.end_type) |= play.to;

        *self.board.get_mut(play.moving_player.other(), play.eaten_type) &= !play.eaten_loc;

        // Castle
        if play.castle_flag {
            let (from_index, to_index): (i32, i32) = match play.to {
                x if x == place(2) => (0, 3),
                x if x == place(6) => (7, 5),
                x if x == place(62) => (63, 61),
                x if x == place(58) => (56, 59),
                _ => {panic!("Tried to castle to an invalid location!")}
            };
            self.board.move_piece(play.moving_player, PieceType::ROOK, place(from_index), place(to_index));
        }

        // En passant
        if play.en_passant_flag {
            let (from_x, from_y) = place_to_coord(play.from);
            let (_, to_y) = place_to_coord(play.to);
            self.board.en_passant_square = place(coord_to_index((from_x, (from_y + to_y) / 2)));
        } else {
            self.board.en_passant_square = 0;
        }

        self.board.castle_memory = play.castle_memory;
        self.current_player = play.moving_player.other();

        self.history.push(play)
    }

    fn undo_move(&mut self) -> Self::T {
        todo!()
    }

    fn game_state(&self) -> GameState {
        todo!()
    }

    fn console_draw(&self) {
        todo!()
    }
}
