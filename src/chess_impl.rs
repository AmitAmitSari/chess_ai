use std::cmp::{max, min};
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::two_player_game::{Game, Player, GameState, Scored};
use crate::bit_help::{index_to_place, place_to_coord, coord_to_index, Dir, index, iter_index, iter_place, ray_until_blocker};
use crate::two_player_game::Player::{PLAYER1, PLAYER2};
use crate::chess_impl::PieceType::{PAWN, KNIGHT, BISHOP, QUEEN, ROOK, KING};
use std::iter::Copied;
use std::slice::Iter;
use crate::move_generation::MoveTables;
use crate::print_u64;
use crate::two_player_game::GameState::PLAYING;
use crate::move_generation::MOVE_TABLE;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PieceType { PAWN = 0, KNIGHT = 1, BISHOP = 2, ROOK = 3, QUEEN = 4, KING = 5 }

impl PieceType {
    pub fn all() -> Copied<Iter<'static, PieceType>> {
        static ALL_PIECES: [PieceType; 6] = [PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
        return ALL_PIECES.iter().copied();
    }
}

impl TryFrom<usize> for PieceType {
    type Error = usize;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        PieceType::all().nth(value).map_or(Result::Err(value), |p| Result::Ok(p))
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
static QUEENSIDE_ROOKS: [u64; 2] = [1 << 7, 1 << 63];

// White is on top.
// The least significant bit is top left. going over the board rows first.
#[derive(Clone)]
pub struct BoardState {
    // Array of two players, with an int per piece type.
    piece_state: [[u64; 6]; 2],

    // Lit bits haven't moved.
    castle_memory: u64,

    // The square the pawn passed over. 0 for not applicable
    en_passant_square: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: u64,
    pub to: u64,
    start_type: PieceType,
    pub end_type: PieceType,

    eaten_type: PieceType,
    // Set to 0 so that nothing is eaten.
    pub eaten_loc: u64,
}

impl Move {
    pub fn hash(&self) -> u64 {
        self.from.trailing_zeros() as u64 |
            (self.to.trailing_zeros() as u64) << 8 |
            (self.start_type as u64) << 16 |
            (self.end_type as u64) << 24 |
            (self.eaten_type as u64) << 32 |
            (self.eaten_loc.trailing_zeros() as u64) << 40
    }

    pub fn serialize(&self) -> String {
        let f = place_to_coord(self.from);
        let t = place_to_coord(self.to);
        let eaten = if self.eaten_loc != 0 {place_to_coord(self.eaten_loc)} else {(-1, -1)};
        let end_type = self.end_type as usize;
        format!("{} {} {} {} {} {} {}", f.0, f.1, t.0, t.1, eaten.0, eaten.1, end_type)
    }
}

fn place_to_letters(place: u64) -> String {
    let (x, y) = place_to_coord(place);
    let lett: String = "hgfedcba".chars().map(|c| c.to_string()).nth(x as usize).unwrap();
    let num: String = "12345678".chars().map(|c| c.to_string()).nth(y as usize).unwrap();
    return lett + &num;
}


impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut res: String = String::new();
        res += &(place_to_letters(self.from) + &place_to_letters(self.to));
        if self.start_type != self.end_type {
            res += ["p", "n", "b", "r", "q", "k"][self.end_type as usize];
        }

        f.write_str(&res)
    }
}

#[derive(Clone)]
pub struct Chess {
    current_player: Player,
    board: BoardState,
    history: Vec<(Move, u64, u64)>,
}

impl Chess {

    pub fn setup_fen_string(&mut self, fen: &str) {

        self.history.clear();

        self.board.piece_state = [[0; 6]; 2];

        let parts: Vec<&str> = fen.split(' ').collect();

        // Pieces
        let mut index = 64_usize;
        for char in parts[0].chars() {
            match char {
                l if "pnbrqk".find(l).is_some() => {
                    index -= 1;
                    *self.board.get_mut(PLAYER2, PieceType::try_from("pnbrqk".find(l).unwrap()).unwrap()) |= index_to_place(index);

                }
                l if "PNBRQK".find(l).is_some() => {
                    index -= 1;
                    *self.board.get_mut(PLAYER1, PieceType::try_from("PNBRQK".find(l).unwrap()).unwrap()) |= index_to_place(index);
                }
                l if "12345678".find(l).is_some() => {
                    index -= "12345678".find(l).unwrap() + 1
                }
                l if l == '/' => { }
                l => {
                    println!("Unexpected char in fen string, {}", l);
                    panic!()
                }
            }
        }

        // next to move
        match parts[1] {
            l if l == "w" => {
                self.current_player = PLAYER1;
            }
            l if l == "b" => {
                self.current_player = PLAYER2
            }
            l => {
                println!("Unexpected char as next player, {}", l);
                panic!()
            }
        }

        self.board.castle_memory = 0;

        // castling
        for (l, place, player) in [('K', KINGSIDE_ROOKS[0], 0), ('k', KINGSIDE_ROOKS[1], 1), ('Q', QUEENSIDE_ROOKS[0], 0), ('q', QUEENSIDE_ROOKS[1], 1)].iter().copied() {
            if parts[2].contains(l) {
                self.board.castle_memory |= place;
                self.board.castle_memory |= KING_PLACES[player];

            }
        }

        // En passant square
        if parts[3] != "-" {
            let x = "hgfedcba".find(&parts[3][0..1]).unwrap() as i32;
            let y = "12345678".find(&parts[3][1..2]).unwrap() as i32;
            self.board.en_passant_square = index_to_place(coord_to_index((x, y)))
        } else {
            self.board.en_passant_square = 0;
        }
    }

    pub fn get_fen_string(&self) -> String {
        let mut res = &mut "".to_owned();

        let info_to_char = [["P", "N", "B", "R", "Q", "K"], ["p", "n", "b", "r", "q", "k"]];

        // Pieces
        for y in (0..8).rev() {
            let mut empty = 0;

            for x in (0..8).rev() {
                let char = self.board.type_at(index_to_place(coord_to_index((x, y)))).map(|(p, t)| info_to_char[p as usize][t as usize]);
                match char {
                    None => {
                        empty += 1
                    }
                    Some(c) => {
                        if empty != 0 {
                            *res += &empty.to_string();
                            empty = 0;
                        }

                        *res += c;
                    }
                }
            }

            if empty != 0 {
                *res += &empty.to_string();

            }
            if y != 0 {
                *res += "/"
            }
        }

        // Next player
        *res += if self.current_player == PLAYER1 { " w " } else { " b " };

        // Castle rights
        let castle_part = &mut "".to_owned();
        for (i, k, q) in [(0, "K", "Q"), (1, "k", "q")].iter().copied() {
            if self.board.castle_memory & KING_PLACES[i] != 0 {
                if self.board.castle_memory & KINGSIDE_ROOKS[i] != 0 {
                    *castle_part += k;
                }
                if self.board.castle_memory & QUEENSIDE_ROOKS[i] != 0 {
                    *castle_part += q;
                }
            }
        }
        if castle_part == "" {
            *castle_part += &"-";
        }
        *res += castle_part;
        *res += &" ";

        // en passant
        if self.board.en_passant_square != 0 {
            *res += &place_to_letters(self.board.en_passant_square);
        } else {
            *res += "-";
        }

        // bogus half-turn counter
        *res += " 0 ";

        // fullturn counter
        *res += &(self.history.len() / 2).to_string();

        res.to_string()
    }

    pub fn get_game_len(&self) -> usize {
        self.history.len()
    }

    pub fn all_pieces(&self) -> Vec<(i32, i32, Player, PieceType)> {
        let mut res = Vec::new();
        for x in 0..8 {
            for y in 0..8 {
                let here = self.board.type_at(index_to_place(coord_to_index((x, y))));
                if let Some((player, piece)) = here {
                    res.push((x, y, player, piece))
                }
            }
        }
        res
    }

    fn castle_rook_move(king_end_location: u64) -> (u64, u64) {
        let (from_index, to_index): (usize, usize) = match index(king_end_location) {
            x if x == 1 => (0, 2),
            x if x == 5 => (7, 4),
            x if x == 61 => (63, 60),
            x if x == 57 => (56, 58),
            x => { panic!("Tried to castle to an invalid location: {}", x) }
        };
        (index_to_place(from_index), index_to_place(to_index))
    }

    fn add_king_moves(&self, possible_moves: &mut Vec<Move>) -> (u64, u64, u64) {
        // Return (king_danger_squares, checkers, push_mask)
        let king_place = self.board.get(self.current_player, KING);
        let king_danger_squares = MOVE_TABLE.get_king_danger_squares(&self.board, self.current_player);
        let king_moves = MOVE_TABLE.get_king_moves(index(king_place))
            & !self.board.occupancy(self.current_player)
            & !king_danger_squares;
        let mut push_mask: u64 = !0;

        self.add_moves(possible_moves, king_place, king_moves, KING, self.board.occupancy(self.current_player.other()));

        let checkers = self.get_checkers();
        if checkers != 0 {
            if checkers.count_ones() > 1 {
                return (king_danger_squares, checkers, 0);
            }
            push_mask = MOVE_TABLE.get_ray(index(king_place), index(checkers))
        } else {
            self.add_castle_moves(possible_moves, checkers, king_danger_squares);
        }

        (king_danger_squares, checkers, push_mask)
    }

    fn add_castle_moves(&self, possible_moves: &mut Vec<Move>, checkers: u64, king_danger: u64) {
        let king_place = self.board.get(self.current_player, KING);
        let rook_board = self.board.get(self.current_player, ROOK);

        let occ = self.board.all_occupancy();
        let cur_player_index = self.current_player as usize;

        if checkers == 0 && self.board.castle_memory & KING_PLACES[cur_player_index] != 0 {
            let mut clear = MOVE_TABLE.get_ray(index(king_place), index(KINGSIDE_ROOKS[cur_player_index])) & (king_danger | occ);
            if clear == 0 && self.board.castle_memory & KINGSIDE_ROOKS[cur_player_index] & rook_board != 0 {
                possible_moves.push(Move {
                    from: king_place,
                    to: king_place >> 2,
                    start_type: KING,
                    end_type: KING,
                    eaten_type: PieceType::PAWN,
                    eaten_loc: 0,
                })
            }


            clear = MOVE_TABLE.get_ray(index(king_place), index(QUEENSIDE_ROOKS[cur_player_index]) - 1) & (king_danger | occ);
            clear |= MOVE_TABLE.get_ray(index(QUEENSIDE_ROOKS[cur_player_index]), index(king_place)) & occ;
            if clear == 0 && self.board.castle_memory & QUEENSIDE_ROOKS[cur_player_index] & rook_board != 0 {
                possible_moves.push(Move {
                    from: king_place,
                    to: king_place << 2,
                    start_type: KING,
                    end_type: KING,
                    eaten_type: PieceType::PAWN,
                    eaten_loc: 0,
                })
            }
        }
    }

    fn add_pinned_moves(&self, possible_moves: &mut Vec<Move>, capture_mask: u64, push_mask: u64) -> u64 {
        // Returns a board of pinned pieces.
        let enemy = self.current_player.other();
        let my_occ = self.board.occupancy(self.current_player);
        let enemy_occ = self.board.occupancy(enemy);
        let occ = my_occ | enemy_occ;
        let king_index = index(self.board.get(self.current_player, KING));

        let enemy_rooks = self.board.get(enemy, ROOK) | self.board.get(enemy, QUEEN);
        let enemy_bishops = self.board.get(enemy, BISHOP) | self.board.get(enemy, QUEEN);

        let mut pinned = 0_u64;
        let mut pinners = 0_u64;
        pinners |= MOVE_TABLE.get_rook_moves(king_index, enemy_occ) & enemy_rooks;
        pinners |= MOVE_TABLE.get_bishop_moves(king_index, enemy_occ) & enemy_bishops;

        for i in iter_index(pinners) {
            let pin = MOVE_TABLE.get_ray(king_index, i) & my_occ;
            if pin.count_ones() == 1 {
                let pin_space = index_to_place(i) | MOVE_TABLE.get_ray(king_index, i);
                let (_, piece_type) = self.board.type_at(pin).unwrap();
                let moves = MOVE_TABLE.get_moves(index(pin), self.current_player, piece_type, occ)
                    & pin_space & (capture_mask | push_mask) & !my_occ;


                self.add_moves(possible_moves, pin, moves, piece_type, enemy_occ);
                pinned |= pin;
            }
        }

        pinned
    }

    fn add_en_passant_captures(&self, possible_moves: &mut Vec<Move>, capture_mask: u64, push_mask: u64, pinned: u64) {
        if self.board.en_passant_square != 0 {
            let king_index = index(self.board.get(self.current_player, KING));

            let eaten = MOVE_TABLE.get_pawn_moves(self.current_player.other(), index(self.board.en_passant_square), 0);

            if (eaten & capture_mask) | (self.board.en_passant_square & push_mask) != 0 {
                let pawns = MOVE_TABLE.get_pawn_captures(self.current_player.other(), index(self.board.en_passant_square), self.board.get(self.current_player, PAWN));
                for i in iter_index(pawns) {
                    // Check pawn isn't pinned. Or is pinned in the right direction.
                    if (index_to_place(i) & pinned != 0) && MOVE_TABLE.get_ray(king_index, index(self.board.en_passant_square)) & index_to_place(i) == 0 {
                        continue;
                    }
                    // Check there is no discovered check from two pawn disappearing from a row.
                    if king_index / 8 == (i / 8) {
                        let no_pawn_occ = self.board.all_occupancy() & !eaten & !index_to_place(i);
                        if MOVE_TABLE.get_rook_moves(king_index, no_pawn_occ) & self.board.get(self.current_player.other(), ROOK) != 0 {
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

        checkers |= MOVE_TABLE.get_rook_moves(king_index, occ) & (self.board.get(enemy, QUEEN) | self.board.get(enemy, ROOK));
        checkers |= MOVE_TABLE.get_bishop_moves(king_index, occ) & (self.board.get(enemy, QUEEN) | self.board.get(enemy, BISHOP));
        checkers |= MOVE_TABLE.get_knight_moves(king_index) & self.board.get(enemy, KNIGHT);

        checkers |= MOVE_TABLE.get_pawn_captures(self.current_player, king_index, self.board.get(enemy, PAWN));

        checkers
    }

    fn add_moves(&self, possible_moves: &mut Vec<Move>, from: u64, to_options: u64, piece_type: PieceType, enemy_occ: u64) {
        for to in iter_place(to_options) {
            let (eaten_loc, eaten_type) = if enemy_occ & to != 0 {
                self.board.type_at(to).map_or((0, PAWN), |x| (to, x.1))
            } else {
                (0, PAWN)
            };
            if piece_type == PAWN && (index(to) / 8) % 7 == 0 {
                for end_type in [QUEEN, ROOK, BISHOP, KNIGHT].iter().cloned() {
                    possible_moves.push(Move {
                        from,
                        to,
                        start_type: piece_type,
                        end_type,
                        eaten_type,
                        eaten_loc,
                    })
                }
            } else {
                let m = Move {
                    from,
                    to,
                    start_type: piece_type,
                    end_type: piece_type,
                    eaten_type,
                    eaten_loc,
                };
                possible_moves.push(m);
            }
        }
    }
}

impl Game for Chess {
    type MoveType = Move;

    fn new() -> Self {
        let mut chess = Chess {
            current_player: PLAYER1,
            board: BoardState {
                piece_state: [[0; 6]; 2],
                castle_memory: 0,
                en_passant_square: 0,
            },
            history: vec![],
        };
        chess.setup_new_game();
        chess
    }

    fn setup_new_game(&mut self) {
        self.setup_fen_string("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }

    fn current_player(&self) -> Player {
        return self.current_player;
    }

    fn possible_moves(&self) -> Vec<Self::MoveType> {
        let mut possible_moves = Vec::with_capacity(50);

        let (king_danger, checkers, push_mask) = self.add_king_moves(&mut possible_moves);

        if checkers.count_ones() > 1 {
            return possible_moves;
        }

        let pinned = self.add_pinned_moves(&mut possible_moves, checkers, push_mask);

        self.add_en_passant_captures(&mut possible_moves, checkers, push_mask, pinned);

        let my_occ = self.board.occupancy(self.current_player);
        let can_move = my_occ & !pinned & !self.board.get(self.current_player, KING);
        let blockers = self.board.all_occupancy();
        let enemy_occ = blockers & !my_occ;
        for p in iter_place(can_move) {
            let (_, piece_type) = self.board.type_at(p).unwrap();
            let moves = MOVE_TABLE.get_moves(index(p), self.current_player, piece_type, blockers)
                & (checkers | push_mask) & !my_occ;
            self.add_moves(&mut possible_moves, p, moves, piece_type, enemy_occ);
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
        let prev_castle = self.board.castle_memory;
        let prev_en_passant = self.board.en_passant_square;

        *self.board.get_mut(self.current_player, play.start_type) &= !play.from;
        *self.board.get_mut(self.current_player, play.end_type) |= play.to;

        *self.board.get_mut(self.current_player.other(), play.eaten_type) &= !play.eaten_loc;

        // Castle
        if play.start_type == KING && play.from & (KING_PLACES[0] | KING_PLACES[1]) != 0 {
            let (from_x, _) = place_to_coord(play.from);
            let (to_x, _) = place_to_coord(play.to);
            if (from_x as i32 - to_x as i32).abs() == 2 {
                let (from_index, to_index) = Chess::castle_rook_move(play.to);
                self.board.move_piece(self.current_player, PieceType::ROOK, from_index, to_index);
            }
        }

        // En passant
        self.board.en_passant_square = 0;
        if play.start_type == PAWN {
            let (from_x, from_y) = place_to_coord(play.from);
            let (_, to_y) = place_to_coord(play.to);
            if (from_y as i32 - to_y as i32).abs() == 2 {
                self.board.en_passant_square = index_to_place(coord_to_index((from_x, (from_y + to_y) / 2)));
            }
        }

        self.board.castle_memory = self.board.castle_memory & !play.from;
        self.current_player = self.current_player.other();

        self.history.push((play, prev_castle, prev_en_passant));
    }

    fn undo_move(&mut self) -> Self::MoveType {
        let (play, castle_memory, en_passant_square) = self.history.pop().unwrap();

        self.current_player = self.current_player.other();

        *self.board.get_mut(self.current_player, play.end_type) &= !play.to;
        *self.board.get_mut(self.current_player, play.start_type) |= play.from;

        *self.board.get_mut(self.current_player.other(), play.eaten_type) |= play.eaten_loc;

        // Castle
        if play.start_type == KING && play.from & (KING_PLACES[0] | KING_PLACES[1]) != 0 {
            let (from_x, _) = place_to_coord(play.from);
            let (to_x, _) = place_to_coord(play.to);
            if (from_x as i32 - to_x as i32).abs() == 2 {
                let (from_index, to_index) = Chess::castle_rook_move(play.to);
                self.board.move_piece(self.current_player, PieceType::ROOK, to_index, from_index);
            }
        }

        self.board.castle_memory = castle_memory;
        self.board.en_passant_square = en_passant_square;


        play
    }

    fn game_state(&self) -> GameState {
        // todo: Implement or some shit.
        return PLAYING;
    }

    fn console_draw(&self) {
        for y in -2..8+2 {
            for x in -4..(8+2)*2 {
                if x % 2 != 0 {
                    print!(" ");
                }
                else if y < 0 || y >= 8 || x / 2 < 0 || x / 2 >= 8 {
                    print!("-")
                }
                else if let Some((player, piece_type)) = self.board.type_at(index_to_place(coord_to_index((x / 2, y)))) {
                    let info_to_char = [["P", "N", "B", "R", "Q", "K"], ["p", "n", "b", "r", "q", "k"]];
                    print!("{}", info_to_char[player as usize][piece_type as usize]);
                } else {
                    print!(".");
                }
            }
            println!();
        }
    }
}


impl Scored for Chess {
    type ScoreType = i32;
    const MAX_INFINITY: Self::ScoreType = i32::MAX;
    const MIN_INFINITY: Self::ScoreType = i32::MIN;
    const MAX_SCORE: Self::ScoreType = 1000;
    const NEUTRAL_SCORE: Self::ScoreType = 0;
    const MIN_SCORE: Self::ScoreType = -1000;

    fn get_score(&self) -> Self::ScoreType {
        let mut game_phase = 0;
        let mut mg_score = 0;
        let mut eg_score = 0;

        /* evaluate each piece */
        for &player in [PLAYER1, PLAYER2].iter() {
            let mult = player as i32 * -2 + 1;
            for piece_type in PieceType::all() {
                // todo: Try piece_type as usize here.
                for i in iter_index(self.board.get(player, piece_type)) {
                    let fi = i ^ ( 56 * (1 - player as usize));
                    mg_score += mult * (MG_TABLE[piece_type as usize][fi] + MG_VALUE[piece_type as usize]);
                    eg_score += mult * (EG_TABLE[piece_type as usize][fi] + EG_VALUE[piece_type as usize]);
                    game_phase += GAMEPHASE_INC[piece_type as usize];
                }
            }
        }

        /* tapered eval */
        let mg_phase = min(game_phase, 24);
        let eg_phase = 24 - mg_phase;
        return (mg_score * mg_phase + eg_score * eg_phase) / 24;
    }

}


static MG_VALUE: [i32; 6] = [ 82, 337, 365, 477, 1025,  0];
static EG_VALUE: [i32; 6] = [ 94, 281, 297, 512,  936,  0];

/* piece/sq tables */
/* values from Rofchade: http://www.talkchess.com/forum3/viewtopic.php?f=2&t=68311&start=19 */

static MG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

static EG_PAWN_TABLE: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

static MG_KNIGHT_TABLE: [i32; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

static EG_KNIGHT_TABLE: [i32; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

static MG_BISHOP_TABLE: [i32; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

static EG_BISHOP_TABLE: [i32; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

static MG_ROOK_TABLE: [i32; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

static EG_ROOK_TABLE: [i32; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

static MG_QUEEN_TABLE: [i32; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

static EG_QUEEN_TABLE: [i32; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

static MG_KING_TABLE: [i32; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

static EG_KING_TABLE: [i32; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

static MG_TABLE: [[i32; 64]; 6] =
[
    MG_PAWN_TABLE,
    MG_KNIGHT_TABLE,
    MG_BISHOP_TABLE,
    MG_ROOK_TABLE,
    MG_QUEEN_TABLE,
    MG_KING_TABLE
];

static EG_TABLE: [[i32; 64]; 6] =
[
    EG_PAWN_TABLE,
    EG_KNIGHT_TABLE,
    EG_BISHOP_TABLE,
    EG_ROOK_TABLE,
    EG_QUEEN_TABLE,
    EG_KING_TABLE
];

static GAMEPHASE_INC: [i32; 6] = [0,1,1,2,4,0];

