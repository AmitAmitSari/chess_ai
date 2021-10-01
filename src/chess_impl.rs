use crate::two_player_game::{Game, Player, GameState};
use crate::bit_help::{index_to_place, place_to_coord, coord_to_index, Dir};
use crate::two_player_game::Player::{PLAYER1, PLAYER2};
use crate::chess_impl::PieceType::{PAWN, KNIGHT, BISHOP, QUEEN, ROOK, KING};
use std::iter::Copied;
use std::slice::Iter;

#[derive(Copy, Clone, PartialEq)]
pub enum PieceType { PAWN = 0, KNIGHT = 1, BISHOP = 2, ROOK = 3, QUEEN = 4, KING = 5 }

impl PieceType {
    pub fn all() -> Copied<Iter<'static, PieceType>> {
        static all_pieces: [PieceType; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        return all_pieces.iter().copied();
    }
}

impl Player {
    pub fn dir(&self, dir: Dir) -> Dir {
        match *self {
            PLAYER2 => dir,
            PLAYER1 => {
                match dir {
                    Dir::North => Dir::South,
                    Dir::South => Dir::North,
                    Dir::East => Dir::West,
                    Dir::West => Dir::East,
                    Dir::NorthEast => Dir::NorthWest,
                    Dir::NorthWest => Dir::SouthEast,
                    Dir::SouthEast => Dir::NorthWest,
                    Dir::SouthWest => Dir::NorthEast
                }
            }
        }
    }
}


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

    pub fn get(&self, player: Player, piece_type: PieceType) -> u64 {
        self.piece_state[player as usize][piece_type as usize]
    }

    fn get_mut(&mut self, player: Player, piece_type: PieceType) -> &mut u64 {
        &mut self.piece_state[player as usize][piece_type as usize]
    }

    fn move_piece(&mut self, player: Player, piece_type: PieceType, from: u64, to: u64) {
        let piece_state = self.get_mut(player, piece_type);
        *piece_state &= !from;
        *piece_state |= to;
    }

    pub fn all_occupancy(&self) -> u64 {
        return self.occupancy(PLAYER1) | self.occupancy(PLAYER2);
    }

    pub fn occupancy(&self, player: Player) -> u64 {
        self.piece_state[player as usize].iter().cloned().reduce(|a, b| a | b).unwrap()
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
    history: Vec<(Move, u64, u64)>
}

impl Chess {
    pub fn castle_rook_move(king_end_location: u64) -> (u64, u64) {
        let (from_index, to_index): (i32, i32) = match king_end_location {
            x if x == index_to_place(2) => (0, 3),
            x if x == index_to_place(6) => (7, 5),
            x if x == index_to_place(62) => (63, 61),
            x if x == index_to_place(58) => (56, 59),
            _ => { panic!("Tried to castle to an invalid location!") }
        };
        (index_to_place(from_index), index_to_place(to_index))
    }
}

impl Game for Chess {
    type T = Move;

    fn current_player(&self) -> Player {
        return self.current_player;
    }

    fn possible_moves(&self) -> Vec<Self::T> {
        let mut possible_moves = vec![];
        possible_moves.reserve(40);


        possible_moves
    }

    fn do_move(&mut self, play: Self::T) {

        *self.board.get_mut(play.moving_player, play.start_type) &= !play.from;
        *self.board.get_mut(play.moving_player, play.end_type) |= play.to;

        *self.board.get_mut(play.moving_player.other(), play.eaten_type) &= !play.eaten_loc;

        // Castle
        if play.castle_flag {
            let (from_index, to_index) = Chess::castle_rook_move(play.to);
            self.board.move_piece(play.moving_player, PieceType::ROOK, from_index, to_index);
        }

        // En passant
        if play.en_passant_flag {
            let (from_x, from_y) = place_to_coord(play.from);
            let (_, to_y) = place_to_coord(play.to);
            self.board.en_passant_square = index_to_place(coord_to_index((from_x, (from_y + to_y) / 2)));
        } else {
            self.board.en_passant_square = 0;
        }

        self.board.castle_memory = play.castle_memory;
        self.current_player = play.moving_player.other();

        self.history.push((play, self.board.castle_memory, self.board.en_passant_square));
    }

    fn undo_move(&mut self) -> Self::T {
        let (play, castle_memory, en_passant_square) = self.history.pop().unwrap();

        *self.board.get_mut(play.moving_player, play.end_type) &= !play.to;
        *self.board.get_mut(play.moving_player, play.start_type) |= play.from;

        *self.board.get_mut(play.moving_player.other(), play.eaten_type) |= play.eaten_loc;

        // Castle
        if play.castle_flag {
            let (from_index, to_index) = Chess::castle_rook_move(play.to);
            self.board.move_piece(play.moving_player, PieceType::ROOK, to_index, from_index);
        }

        self.board.castle_memory = castle_memory;
        self.board.en_passant_square = en_passant_square;
        self.current_player = play.moving_player.other();

        play
    }

    fn game_state(&self) -> GameState {
        todo!()
    }

    fn console_draw(&self) {
        todo!()
    }
}
