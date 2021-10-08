use crate::two_player_game::{Game, Player, GameState};
use crate::bit_help::{index_to_place, place_to_coord, coord_to_index, Dir, index, iter_index, iter_place};
use crate::two_player_game::Player::{PLAYER1, PLAYER2};
use crate::chess_impl::PieceType::{PAWN, KNIGHT, BISHOP, QUEEN, ROOK, KING};
use std::iter::Copied;
use std::slice::Iter;
use crate::move_generation::MoveTables;
use crate::two_player_game::GameState::PLAYING;

#[derive(Copy, Clone, PartialEq)]
pub enum PieceType { PAWN = 0, KNIGHT = 1, BISHOP = 2, ROOK = 3, QUEEN = 4, KING = 5 }

impl PieceType {
    pub fn all() -> Copied<Iter<'static, PieceType>> {
        static ALL_PIECES: [PieceType; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        return ALL_PIECES.iter().copied();
    }
}

impl Player {
    pub fn dir(&self, dir: Dir) -> Dir {
        match *self {
            PLAYER2 => dir,
            PLAYER1 => dir.flip()
        }
    }
}

static KING_PLACES: [u64; 2] = [1 << 3, 1 << (63 - 4)];
static KINGSIDE_ROOKS: [u64; 2] = [1, 1 << (63 - 7)];
static QUEENSIDE_ROOKS: [u64; 2] = [1 << 7, 63];

// White is on top.
// The least significant bit is top left. going over the board rows first.
pub struct BoardState {
    // Array of two players, with an int per piece type.
    piece_state: [[u64; 6]; 2],

    // Lit bits haven't moved.
    castle_memory: u64,

    // The square the pawn passed over. 0 for not applicable
    en_passant_square: u64
}

impl BoardState {
    pub fn type_at(&self, place: u64) -> Option<(Player, PieceType)> {
        for player in [PLAYER1, PLAYER2].iter() {
            for piece_type in PieceType::all() {
                if self.get(*player, piece_type) & place != 0 {
                    return Some((*player, piece_type));
                }
            }
        }
        return None;
    }

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
    // Move, castle memory and en passant square after move.. todo: Change to before move for easier undo
    history: Vec<(Move, u64, u64)>,
    move_tables: Box<MoveTables>
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

    fn add_king_moves(&self, possible_moves: &mut Vec<Move>) -> (u64, u64, u64) {
        // Return (king_danger_squares, checkers, push_mask)
        let king_place = self.board.get(self.current_player, KING);
        let king_danger_squares = self.move_tables.get_king_danger_squares(&self.board, self.current_player);
        let king_moves = self.move_tables.get_king_moves(index(king_place) as usize)
            & !self.board.occupancy(self.current_player)
            & !king_danger_squares;
        let mut push_mask: u64 = !0;

        self.add_moves(possible_moves, king_place, king_moves, KING);

        let checkers = self.get_checkers();
        if checkers != 0 {
            if checkers.count_ones() > 1 {
                return (king_danger_squares, checkers, 0)
            }
            push_mask = self.move_tables.get_ray(index(king_place) as usize, index(checkers) as usize)
        } else {
            self.add_castle_moves(possible_moves, checkers, king_danger_squares);
        }

        (king_danger_squares, checkers, push_mask)
    }

    fn add_castle_moves(&self, possible_moves: &mut Vec<Move>, checkers: u64, king_danger: u64) {
        let king_place = self.board.get(self.current_player, KING);

        let occ = self.board.all_occupancy();
        let cur_player_index = self.current_player as usize;

        if checkers == 0 && self.board.castle_memory & KING_PLACES[cur_player_index] != 0 {
            let mut clear = self.move_tables.get_ray(index(king_place) as usize, index(KINGSIDE_ROOKS[cur_player_index]) as usize) & (king_danger | occ);
            if clear == 0 && self.board.castle_memory & KINGSIDE_ROOKS[cur_player_index] != 0 {
                possible_moves.push(Move {
                    from: king_place,
                    to: king_place >> 2,
                    start_type: KING,
                    end_type: KING,
                    eaten_type: PieceType::PAWN,
                    eaten_loc: 0,
                    castle_memory: self.board.castle_memory & !king_place,
                    castle_flag: true,
                    en_passant_flag: false,
                    moving_player: self.current_player
                })
            }


            clear = self.move_tables.get_ray(index(king_place) as usize, index(QUEENSIDE_ROOKS[cur_player_index]) as usize) & (king_danger | occ);
            clear |= self.move_tables.get_rook_moves(index(QUEENSIDE_ROOKS[cur_player_index]) as usize, king_place) & occ;
            if clear == 0 && self.board.castle_memory & QUEENSIDE_ROOKS[cur_player_index] != 0 {
                possible_moves.push(Move {
                    from: king_place,
                    to: king_place << 2,
                    start_type: KING,
                    end_type: KING,
                    eaten_type: PieceType::PAWN,
                    eaten_loc: 0,
                    castle_memory: self.board.castle_memory & !king_place,
                    castle_flag: true,
                    en_passant_flag: false,
                    moving_player: self.current_player
                })
            }
        }
    }

    fn add_pinned_moves(&self, possible_moves: &mut Vec<Move>, capture_mask: u64, push_mask: u64) -> u64 {
        // Returns a board of pinned pieces.
        let occ = self.board.all_occupancy();
        let my_occ = self.board.occupancy(self.current_player);
        let king_index = index(self.board.get(self.current_player, KING)) as usize;
        let enemy = self.current_player.other();
        let enemy_rooks = self.board.get(enemy, ROOK) | self.board.get(enemy, QUEEN);
        let enemy_bishops = self.board.get(enemy, BISHOP) | self.board.get(enemy, QUEEN);

        let mut pinned = 0_u64;
        let mut pinners = 0_u64;
        pinners |= self.move_tables.get_rook_moves(king_index, enemy_rooks) & enemy_rooks;
        pinners |= self.move_tables.get_bishop_moves(king_index, enemy_bishops) & enemy_bishops;

        for i in iter_index(pinners) {
            let pin = self.move_tables.get_ray(king_index, i as usize) & my_occ;
            if pin.count_ones() == 1 {
                let pin_space = index_to_place(i) | self.move_tables.get_ray(king_index, i as usize);
                let (_, piece_type) = self.board.type_at(pin).unwrap();
                let moves = self.move_tables.get_moves(index(pin) as usize, self.current_player, piece_type, occ)
                    & pin_space & (capture_mask | push_mask) & !my_occ;


                self.add_moves(possible_moves, pin, moves, piece_type);
                pinned |= pin;
            }
        }

        pinned
    }

    fn add_en_passant_captures(&self, possible_moves: &mut Vec<Move>, capture_mask: u64, push_mask: u64, pinned: u64) {
        if self.board.en_passant_square != 0 {
            let king_index = index(self.board.get(self.current_player, KING)) as usize;

            let eaten = self.move_tables.get_pawn_moves(self.current_player.other(), index(self.board.en_passant_square) as usize, 0);
            if (eaten & capture_mask) | (self.board.en_passant_square & push_mask) != 0 {
                let pawns = self.move_tables.get_pawn_captures(self.current_player.other(), index(self.board.en_passant_square) as usize, self.board.get(self.current_player, PAWN));
                for i in iter_index(pawns) {
                    // Check pawn isn't pinned. Or is pinned in the right direction.
                    if (index_to_place(i) & pinned != 0) && self.move_tables.get_ray(king_index, index(self.board.en_passant_square) as usize) & index_to_place(i) != 0 {
                        continue;
                    }
                    // Check there is no discovered check from two pawn disappearing from a row.
                    if king_index / 8 == (i / 8) as usize {
                        let no_pawn_occ = self.board.all_occupancy() & !eaten & !index_to_place(i);
                        if self.move_tables.get_rook_moves(king_index, no_pawn_occ) & self.board.get(self.current_player.other(), ROOK) != 0 {
                            continue;
                        }
                    }

                    possible_moves.push(Move {
                        from: index_to_place(i),
                        to: self.board.en_passant_square,
                        start_type: PieceType::PAWN,
                        end_type: PieceType::PAWN,
                        eaten_type: PieceType::PAWN,
                        eaten_loc: eaten,
                        castle_memory: self.board.castle_memory,
                        castle_flag: false,
                        en_passant_flag: false,
                        moving_player: self.current_player
                    })

                }
            }
        }
    }

    fn get_checkers(&self) -> u64 {
        let mut checkers = 0;

        let king_index = index(self.board.get(self.current_player, KING));
        let enemy = self.current_player.other();
        let occ = self.board.all_occupancy();

        checkers |= self.move_tables.get_rook_moves(king_index as usize, occ) & (self.board.get(enemy, QUEEN) | self.board.get(enemy, ROOK));
        checkers |= self.move_tables.get_bishop_moves(king_index as usize, occ) & (self.board.get(enemy, QUEEN) | self.board.get(enemy, BISHOP));
        checkers |= self.move_tables.get_knight_moves(king_index as usize) & self.board.get(enemy, KNIGHT);

        checkers |= self.move_tables.get_pawn_captures(self.current_player, king_index as usize, self.board.get(enemy, PAWN));

        checkers
    }

    fn add_moves(&self, possible_moves: &mut Vec<Move>, from: u64, to_options: u64, piece_type: PieceType) {
        for to in iter_place(to_options) {
            let (eaten_loc, eaten_type) = self.board.type_at(to).map(|x| (to, x.1)).unwrap_or((0, PAWN));
            possible_moves.push(Move {
                from,
                to,
                start_type: piece_type,
                end_type: piece_type,
                eaten_type,
                eaten_loc,
                castle_memory: self.board.castle_memory & !from,
                castle_flag: false,
                en_passant_flag: false,
                moving_player: self.current_player
            })
        }

    }
}

impl Game for Chess {
    type MoveType = Move;

    fn new() -> Self {
        let mut chess = Chess {
            current_player: Player::PLAYER1,
            board: BoardState {
                piece_state: [[0; 6]; 2],
                castle_memory: 0,
                en_passant_square: 0
            },
            history: vec![],
            move_tables: Box::new(MoveTables::new())
        };
        chess.setup_new_game();
        chess
    }

    fn setup_new_game(&mut self) {
        self.board.castle_memory = KING_PLACES[0] | KING_PLACES[1] | KINGSIDE_ROOKS[0] | KINGSIDE_ROOKS[1] | QUEENSIDE_ROOKS[0] | QUEENSIDE_ROOKS[1];
        self.board.en_passant_square = 0;

        self.history.clear();
        self.current_player = Player::PLAYER1;

        self.board.piece_state = [[0; 6]; 2];

        // PAWNS
        for i in 8..16 {
            *self.board.get_mut(PLAYER1, PAWN) |= index_to_place(i);
        }
        *self.board.get_mut(PLAYER1, PAWN) |= self.board.get(PLAYER1, PAWN) << (8 * 5);

        *self.board.get_mut(PLAYER1, ROOK) |= index_to_place(0) | index_to_place(7);
        *self.board.get_mut(PLAYER1, KNIGHT) |= index_to_place(1) | index_to_place(6);
        *self.board.get_mut(PLAYER1, BISHOP) |= index_to_place(2) | index_to_place(5);
        *self.board.get_mut(PLAYER1, QUEEN) |= index_to_place(4);
        *self.board.get_mut(PLAYER1, KING) |= index_to_place(3);

        for piece_type in [ROOK, KNIGHT, BISHOP, QUEEN, KING].iter() {
            *self.board.get_mut(PLAYER2, *piece_type) = self.board.get(PLAYER1, *piece_type) << (8 * 7)
        }

    }

    fn current_player(&self) -> Player {
        return self.current_player;
    }

    fn possible_moves(&self) -> Vec<Self::MoveType> {
        let mut possible_moves = vec![];
        possible_moves.reserve(40);

        let (king_danger, checkers, push_mask) = self.add_king_moves(&mut possible_moves);

        if checkers.count_ones() > 1 {
            return possible_moves;
        }

        let pinned = self.add_pinned_moves(&mut possible_moves, checkers, push_mask);

        self.add_en_passant_captures(&mut possible_moves, checkers, push_mask, pinned);

        let can_move = self.board.occupancy(self.current_player) & !pinned & !self.board.get(self.current_player, KING);
        let blockers = self.board.all_occupancy();
        let my_occ = self.board.occupancy(self.current_player);
        for p in iter_place(can_move) {
            let (_, piece_type) = self.board.type_at(p).unwrap();
            let moves = self.move_tables.get_moves(index(p) as usize, self.current_player, piece_type, blockers)
                & (checkers | push_mask) & !my_occ;
            self.add_moves(&mut possible_moves, p, moves, piece_type);
        }

        // Find king moves:
            // Find king danger squares
            // Find my occupancy
            // Calc king moves

        // Handle check - If I am in check I can only:
            // move the king with the moves I found,
            // If there is only one checker:
                // Eat the checking piece - Generates capture_mask
                // Block the checking piece - Generates push mask
            // All future moves need to & (capture_mask | push mask)

        // Find castles

        // Find pinned pieces and their moves.
            // Find pinners
                // find rooks at places that are rook move from the king
                // find bishops that are at bishop moves from the king
            // Find pinners with exactly one piece in the pin space.
            // That piece can only move in the pin space.

        // en passant moves
            // if there is an en passant square
            // en passant is possible if the the pawn square is in the capture mask - or the move square is in the push mask
            // if the pawn in pinned, it can only move if it is in the ray between my king and the move space.
            // En passant removes two pieces from the pawn row, this can create a discovered check if the king is on the same row.


        // regular moves :)


        possible_moves
    }

    fn do_move(&mut self, play: Self::MoveType) {

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

    fn undo_move(&mut self) -> Self::MoveType {
        let (play, castle_memory, en_passant_square) = self.history.pop().unwrap();

        *self.board.get_mut(play.moving_player, play.end_type) &= !play.to;
        *self.board.get_mut(play.moving_player, play.start_type) |= play.from;

        *self.board.get_mut(play.moving_player.other(), play.eaten_type) |= play.eaten_loc;

        // Castle
        if play.castle_flag {
            let (from_index, to_index) = Chess::castle_rook_move(play.to);
            self.board.move_piece(play.moving_player, PieceType::ROOK, to_index, from_index);
        }

        // todo: This is wrong, change move castle memory to be the one before the move
        self.board.castle_memory = castle_memory;
        self.board.en_passant_square = en_passant_square;
        self.current_player = play.moving_player.other();

        play
    }

    fn game_state(&self) -> GameState {
        // todo: Implement or some shit.
        return PLAYING;
    }

    fn console_draw(&self) {
        for y in 0..8 {
            for x in 0..8 {
                if let Some((player, piece_type)) = self.board.type_at(index_to_place(coord_to_index((x, y)))) {
                    let info_to_char = [["P", "N", "B", "R", "Q", "K"], ["p", "n", "b", "r", "Q", "k"]];
                    print!("{}", info_to_char[player as usize][piece_type as usize]);
                } else {
                    print!(" ");
                }
            }
            println!();
        }
    }
}
