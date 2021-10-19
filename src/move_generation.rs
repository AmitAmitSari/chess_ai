use std::detect::__is_feature_detected::adx;
use crate::bit_help::{Dir, ray, ray_until_blocker, index_to_place, iter_index, _ray, index, index_to_coord, coord_to_index};
use crate::two_player_game::Player::{PLAYER1, PLAYER2};
use crate::two_player_game::Player;
use crate::chess_impl::{BoardState, PieceType};
use crate::chess_impl::PieceType::PAWN;
use crate::print_u64;

// These where generated such that for each square - multiplying the blockerboard with the magic for that square gives a unique set of the most significant X bits.
const _ROOK_MAGICS: [u64; 64] = [
    0xa8002c000108020_u64, 0x6c00049b0002001_u64, 0x100200010090040_u64, 0x2480041000800801_u64, 0x280028004000800_u64,
    0x900410008040022_u64, 0x280020001001080_u64, 0x2880002041000080_u64, 0xa000800080400034_u64, 0x4808020004000_u64,
    0x2290802004801000_u64, 0x411000d00100020_u64, 0x402800800040080_u64, 0xb000401004208_u64, 0x2409000100040200_u64,
    0x1002100004082_u64, 0x22878001e24000_u64, 0x1090810021004010_u64, 0x801030040200012_u64, 0x500808008001000_u64,
    0xa08018014000880_u64, 0x8000808004000200_u64, 0x201008080010200_u64, 0x801020000441091_u64, 0x800080204005_u64,
    0x1040200040100048_u64, 0x120200402082_u64, 0xd14880480100080_u64, 0x12040280080080_u64, 0x100040080020080_u64,
    0x9020010080800200_u64, 0x813241200148449_u64, 0x491604001800080_u64, 0x100401000402001_u64, 0x4820010021001040_u64,
    0x400402202000812_u64, 0x209009005000802_u64, 0x810800601800400_u64, 0x4301083214000150_u64, 0x204026458e001401_u64,
    0x40204000808000_u64, 0x8001008040010020_u64, 0x8410820820420010_u64, 0x1003001000090020_u64, 0x804040008008080_u64,
    0x12000810020004_u64, 0x1000100200040208_u64, 0x430000a044020001_u64, 0x280009023410300_u64, 0xe0100040002240_u64,
    0x200100401700_u64, 0x2244100408008080_u64, 0x8000400801980_u64, 0x2000810040200_u64, 0x8010100228810400_u64,
    0x2000009044210200_u64, 0x4080008040102101_u64, 0x40002080411d01_u64, 0x2005524060000901_u64, 0x502001008400422_u64,
    0x489a000810200402_u64, 0x1004400080a13_u64, 0x4000011008020084_u64, 0x26002114058042_u64
];

const _BISHOP_MAGICS: [u64; 64] = [
    0x89a1121896040240_u64, 0x2004844802002010_u64, 0x2068080051921000_u64, 0x62880a0220200808_u64, 0x4042004000000_u64,
    0x100822020200011_u64, 0xc00444222012000a_u64, 0x28808801216001_u64, 0x400492088408100_u64, 0x201c401040c0084_u64,
    0x840800910a0010_u64, 0x82080240060_u64, 0x2000840504006000_u64, 0x30010c4108405004_u64, 0x1008005410080802_u64,
    0x8144042209100900_u64, 0x208081020014400_u64, 0x4800201208ca00_u64, 0xf18140408012008_u64, 0x1004002802102001_u64,
    0x841000820080811_u64, 0x40200200a42008_u64, 0x800054042000_u64, 0x88010400410c9000_u64, 0x520040470104290_u64,
    0x1004040051500081_u64, 0x2002081833080021_u64, 0x400c00c010142_u64, 0x941408200c002000_u64, 0x658810000806011_u64,
    0x188071040440a00_u64, 0x4800404002011c00_u64, 0x104442040404200_u64, 0x511080202091021_u64, 0x4022401120400_u64,
    0x80c0040400080120_u64, 0x8040010040820802_u64, 0x480810700020090_u64, 0x102008e00040242_u64, 0x809005202050100_u64,
    0x8002024220104080_u64, 0x431008804142000_u64, 0x19001802081400_u64, 0x200014208040080_u64, 0x3308082008200100_u64,
    0x41010500040c020_u64, 0x4012020c04210308_u64, 0x208220a202004080_u64, 0x111040120082000_u64, 0x6803040141280a00_u64,
    0x2101004202410000_u64, 0x8200000041108022_u64, 0x21082088000_u64, 0x2410204010040_u64, 0x40100400809000_u64,
    0x822088220820214_u64, 0x40808090012004_u64, 0x910224040218c9_u64, 0x402814422015008_u64, 0x90014004842410_u64,
    0x1000042304105_u64, 0x10008830412a00_u64, 0x2520081090008908_u64, 0x40102000a0a60140_u64,
];

// The number of bits needed to save all options for blockers in this square.
const _ROOK_INDEX_BITS: [i32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

const _BISHOP_INDEX_BITS: [i32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6
];


pub struct MoveTables {

    // All move location for each square - not including board edge
    bishop_masks: [u64; 64],
    rook_masks: [u64; 64],

    // Map from key to moveboard for each square
    bishop_table: Vec<[u64; 1024]>,
    rook_table: Vec<[u64; 4096]>,

    // Non sliding piece masks
    knight_masks: [u64; 64],
    king_masks: [u64; 64],

    rays: [[u64; 64]; 8],
}


impl MoveTables {

    pub fn new() -> MoveTables {
        let mut m = MoveTables {
            bishop_masks: [0; 64],
            rook_masks: [0; 64],
            bishop_table: vec![[0; 1024]; 64],
            rook_table: vec![[0; 4096]; 64],
            knight_masks: [0; 64],
            king_masks: [0; 64],
            rays: [[0; 64]; 8],
        };
        m._init();
        m
    }

    fn _init(&mut self) {
        self._init_bishop_masks();
        self._init_rook_masks();
        self._init_bishop_tables();
        self._init_rook_tables();

        self._init_knight_masks();
        self._init_king_masks();

        self._init_rays();
    }

    pub fn get_moves(&self, index: usize, player: Player, piece_type: PieceType, blockers: u64) -> u64 {
        // Does not include castle or en passant capture

        match piece_type {
            PieceType::PAWN => self.get_pawn_moves(player, index, blockers) | self.get_pawn_captures(player, index, blockers),
            PieceType::KNIGHT => self.get_knight_moves(index),
            PieceType::BISHOP => self.get_bishop_moves(index, blockers),
            PieceType::ROOK => self.get_rook_moves(index, blockers),
            PieceType::QUEEN => self.get_rook_moves(index, blockers) | self.get_bishop_moves(index, blockers),
            PieceType::KING => self.get_king_moves(index)
        }

    }

    pub fn get_bishop_moves(&self, index: usize, blockers: u64) -> u64 {
        let key = ((blockers & self.bishop_masks[index]).wrapping_mul(_BISHOP_MAGICS[index])) >> (64 - _BISHOP_INDEX_BITS[index]);
        self.bishop_table[index][key as usize]
    }

    pub fn get_rook_moves(&self, index: usize, blockers: u64) -> u64 {
        let key = ((blockers & self.rook_masks[index]).wrapping_mul(_ROOK_MAGICS[index])) >> (64 - _ROOK_INDEX_BITS[index]);
        self.rook_table[index][key as usize]
    }

    pub fn get_knight_moves(&self, index: usize) -> u64 {
        self.knight_masks[index]
    }

    pub fn get_king_moves(&self, index: usize) -> u64 {
        self.king_masks[index]
    }

    pub fn get_pawn_moves(&self, player: Player, index: usize, blockers: u64) -> u64 {
        let mut res = 0_u64;
        let (start_row, row_diff) = match player {
            PLAYER1 => (1, 2),
            PLAYER2 => (6, -2)
        };
        let dir = player.dir(Dir::North);

        if let Some(move_one) = dir.mv(index_to_place(index)) {
            res |= move_one & !blockers;
            if res != 0 && index / 8 == start_row {
                res |= index_to_place((index as i32 + row_diff * 8) as usize) & !blockers;
            }
        }
        res
    }

    pub fn get_pawn_captures(&self, player: Player, index: usize, enemy_blockers: u64) -> u64 {
        let mut res = 0_u64;

        for dir in [Dir::NorthEast, Dir::NorthWest].iter() {
            if let Some(diag_place) = player.dir(*dir).mv(index_to_place(index)) {
                res |= diag_place
            }
        }

        res &= enemy_blockers;
        res
    }

    pub fn get_king_danger_squares(&self, board: &BoardState, player: Player) -> u64 {
        let mut king_danger = 0_u64;
        let other = player.other();

        let occ_no_king = board.all_occupancy() & !board.get(player, PieceType::KING);

        for piece_type in PieceType::all() {
            for index in iter_index(board.get(other, piece_type)) {
                if piece_type == PAWN {
                    king_danger |= self.get_pawn_captures(other, index, !0);
                }
                else {
                    king_danger |= self.get_moves(index, other, piece_type, occ_no_king);
                }
            }
        }

        king_danger
    }

    pub fn get_ray(&self, from: usize, to: usize) -> u64 {
        // Not including from and to.
        let mut res = 0;
        for dir in Dir::all() {
            res |= self.rays[dir as usize][from] & self.rays[dir.flip() as usize][to]
        }
        res
    }

    fn _create_blockers_from_index(index: i32, mut mask: u64) -> u64 {
        let mut blockers: u64 = 0;
        let bits = mask.count_ones();
        for i in 0..bits {
            let bit_pos = mask.trailing_zeros();
            mask ^= mask & (!mask + 1);
            if (index & (1 << i)) != 0 {
                blockers |= 1_u64 << bit_pos
            }
        }
        blockers
    }

    fn _init_bishop_masks(&mut self) {
        for index in 0..64 {
            self.bishop_masks[index] |= _ray(index, Dir::NorthEast, 1);
            self.bishop_masks[index] |= _ray(index, Dir::NorthWest, 1);
            self.bishop_masks[index] |= _ray(index, Dir::SouthEast, 1);
            self.bishop_masks[index] |= _ray(index, Dir::SouthWest, 1);
        }
    }

    fn _init_rook_masks(&mut self) {
        for index in 0..64 {
            self.rook_masks[index] |= _ray(index, Dir::North, 1);
            self.rook_masks[index] |= _ray(index, Dir::East, 1);
            self.rook_masks[index] |= _ray(index, Dir::West, 1);
            self.rook_masks[index] |= _ray(index, Dir::South, 1);
        }
    }

    fn _init_bishop_tables(&mut self) {
        for index in 0..64 {
            for blocker_index in 0..1<<_BISHOP_INDEX_BITS[index] {
                let blockers = MoveTables::_create_blockers_from_index(blocker_index as i32, self.bishop_masks[index]);
                let key = (blockers.wrapping_mul(_BISHOP_MAGICS[index])) >> (64 - _BISHOP_INDEX_BITS[index]);
                self.bishop_table[index][key as usize] = MoveTables::_bishop_moves_slow(index, blockers);
            }
        }
    }

    fn _init_rook_tables(&mut self) {
        for index in 0..64 {
            for blocker_index in 0..1<<_ROOK_INDEX_BITS[index] {
                let blockers = MoveTables::_create_blockers_from_index(blocker_index as i32, self.rook_masks[index]);
                let key = (blockers.wrapping_mul(_ROOK_MAGICS[index])) >> (64 - _ROOK_INDEX_BITS[index]);
                self.rook_table[index][key as usize] = MoveTables::_rook_moves_slow(index, blockers);
            }
        }
    }

    fn _init_knight_masks(&mut self) {
        for i in 0..64 {
            let (x, y) = index_to_coord(i);
            for dx in [-2_i32, -1, 1, 2].iter().cloned() {
                for dy in [-2_i32, -1, 1, 2].iter().cloned() {
                    if dx.abs() != dy.abs() {
                        let (nx, ny) = (x + dx, y + dy);
                        if 0 <= nx && nx < 8 && 0 <= ny && ny < 8 {
                            self.knight_masks[i] |= index_to_place(coord_to_index((nx, ny)));
                        }
                    }
                }
            }
        }
    }

    fn _init_king_masks(&mut self) {
        for index in 0..64 {
            for dir in Dir::all() {
                if let Some(m) = dir.mv(index_to_place(index)) {
                    self.king_masks[index] |= m;
                }
            }
        }
    }

    fn _init_rays(&mut self) {
        for dir in Dir::all() {
            for index in 0..64 {
                self.rays[dir as usize][index] = ray(index, dir);
            }
        }
    }

    fn _bishop_moves_slow(index: usize, blockers: u64) -> u64 {
        let mut res = 0_u64;

        res |= ray_until_blocker(index, blockers, Dir::NorthWest);
        res |= ray_until_blocker(index, blockers, Dir::NorthEast);
        res |= ray_until_blocker(index, blockers, Dir::SouthWest);
        res |= ray_until_blocker(index, blockers, Dir::SouthEast);

        res
    }

    fn _rook_moves_slow(index: usize, blockers: u64) -> u64 {
        let mut res = 0_u64;

        res |= ray_until_blocker(index, blockers, Dir::North);
        res |= ray_until_blocker(index, blockers, Dir::South);
        res |= ray_until_blocker(index, blockers, Dir::East);
        res |= ray_until_blocker(index, blockers, Dir::West);

        res
    }
}