use crate::two_player_game::Game;
use crate::two_player_game::Scored;
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::{Duration, Instant};
use ahash::AHashMap;
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use crate::chess_impl::{Chess, Move};
use crate::get_time;

type Cache = HashMap<u64, i32, A>;
type Caches = HashMap<usize, Cache, A>;


pub struct A {

}

pub struct H {
    state: u64
}

impl BuildHasher for A {
    type Hasher = H;

    fn build_hasher(&self) -> Self::Hasher {
        H { state: 0 }
    }
}

impl Hasher for H {
    fn finish(&self) -> u64 {
        return self.state;
    }

    fn write(&mut self, bytes: &[u8]) {
        todo!()
    }

    fn write_u64(&mut self, i: u64) {
        self.state = i;
    }

    fn write_usize(&mut self, i: usize) {
        self.state = i as u64;
    }
}

pub struct MoveResult {
    pub chess_move: Option<<Chess as Game>::MoveType>,
    pub move_from_depth: i32,
}

pub fn get_next_move(game: &mut Chess, depth: i32, max_timestamp_ms: u128) -> MoveResult {

    let mut killer_move_cache: Caches = HashMap::with_capacity_and_hasher(depth as usize, A {});
    let mut m = None;
    let mut total_count = 0;
    let mut move_from_depth = 0;

    let start = Instant::now();
    for i in min(4,depth)..depth+1 {
        let call_count: &mut i32 = &mut 0;
        let ores = _get_next_move(game, i, &mut killer_move_cache, call_count, max_timestamp_ms);
        if let Some(res) = ores {
            if res.0.is_some() {
                m = res.0;
                move_from_depth = i;
            }
            total_count += *call_count;
            let nps = total_count as f64 / start.elapsed().as_secs_f64();
            eprintln!("Depth: {}, Move: {}, Score: {}, CallCount: {}, Total: {}, NPS: {}", i, m.clone().unwrap(), res.1, call_count, total_count, nps as u64);
        }
    }
    // let mut cache = killer_move_cache.iter().collect::<Vec<_>>();
    // cache.sort_unstable_by_key(|(k, v)| *k );
    // for (key, value) in cache {
    //
    //     let mut moves = value.iter().collect::<Vec<_>>();
    //     moves.sort_unstable_by_key(|(_,&v)| -v);
    //     let moves = moves.iter().map(|(m, c)| m.to_string() + ": " + &c.to_string()).collect::<Vec<_>>();
    //     eprintln!("{}: {:?}", key, moves);
    // }
    MoveResult {
        chess_move: m,
        move_from_depth
    }
}

#[inline]
fn move_ordering(m: &Move, killer_move_cache_at_depth: &Cache) -> i32 {
    -max(*killer_move_cache_at_depth.get(&m.hash()).unwrap_or(&0), (m.eaten_loc != 0) as i32 * 10)
}

fn _get_next_move(game: &mut Chess, depth: i32, mut killer_move_cache: &mut Caches, call_count: &mut i32, max_timestamp_ms: u128) -> Option<(Option<<Chess as Game>::MoveType>, <Chess as Scored>::ScoreType)>
{
    let mut rng = rand::thread_rng();
    let mut best_moves = vec![];
    let mut score;
    let mut a = Chess::MIN_INFINITY;
    let mut b = Chess::MAX_INFINITY;

    let mut possible_moves = game.possible_moves();
    *call_count += 1;
    let at_depth = killer_move_cache.entry(game.get_game_len()).or_insert(HashMap::with_hasher(A {}));
    possible_moves.sort_by_cached_key(|m| move_ordering(m, at_depth));

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY + (game.get_game_len() * 100) as <Chess as Scored>::ScoreType;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to, &mut killer_move_cache, call_count, max_timestamp_ms)?;
            let m_ = game.undo_move();
            if move_score >= score {
                if move_score > score {
                    // eprintln!("{}, {}", m_, move_score);
                    best_moves.clear();
                }
                best_moves.push(m_);
                score = move_score
            }
            a = max(a, score);
        }
    } else {
        score = Chess::MAX_INFINITY - (game.get_game_len() * 100) as <Chess as Scored>::ScoreType;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to, &mut killer_move_cache, call_count, max_timestamp_ms)?;
            let m_ = game.undo_move();
            if move_score <= score {
                if move_score < score {
                    // eprintln!("{}, {}", m_, move_score);
                    best_moves.clear();
                }
                best_moves.push(m_);
                score = move_score
            }
            b = min(b, score);
        }
    }

    let i = (0..best_moves.len()).choose(&mut rng);

    let m = i.map(|x| best_moves.swap_remove(x));
    if let Some(m_) = m.clone() {
        *killer_move_cache.entry(game.get_game_len()).or_insert(HashMap::with_hasher(A {})).entry(m_.hash()).or_insert(0) += 1;
    }
    return Some((m, score));
}

pub fn min_max(game: &mut Chess, depth: i32, last_to: u64) -> <Chess as Scored>::ScoreType {
    let mut possible_moves = game.possible_moves();

    if depth <= 0 {
        // Quiescence.
        possible_moves.retain(|m| m.eaten_loc != 0 && m.eaten_loc == last_to);
        if possible_moves.len() == 0 {
            return game.get_score();
        }
    }

    let mut score;

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            score = max(score, min_max(game, depth - 1, to));
            game.undo_move();
        }
    } else {
        score = Chess::MAX_INFINITY;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            score = min(score, min_max(game, depth - 1, to));
            game.undo_move();
        }
    }
    return score;
}

pub fn alpha_beta(game: &mut Chess, depth: i32, mut a: <Chess as Scored>::ScoreType, mut b: <Chess as Scored>::ScoreType, last_to: u64, killer_move_cache: &mut Caches, call_count: &mut i32, max_timestamp_ms: u128) -> Option<<Chess as Scored>::ScoreType>
{
    // if get_time() >= max_timestamp_ms {
    //     return None;
    // }

    let mut possible_moves = game.possible_moves();
    *call_count += 1;

    if depth <= 0 {
        // Quiescence.
        possible_moves.retain(|m| m.eaten_loc != 0 && m.eaten_loc == last_to);
        if possible_moves.len() == 0 {
            return Some(game.get_score());
        }
    } else {
        // Only sort if depth is high enough to be worth it - doesn't help
        let at_depth = killer_move_cache.entry(game.get_game_len()).or_insert(HashMap::with_hasher(A {}));
        possible_moves.sort_by_cached_key(|m| move_ordering(m, at_depth));
    }

    let mut score;

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY + (game.get_game_len() * 100) as <Chess as Scored>::ScoreType;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            let move_score = if depth > 1 || to != 0 {alpha_beta(game, depth - 1, a, b, to, killer_move_cache, call_count, max_timestamp_ms)?} else { game.get_score() };
            score = max(score, move_score);
            let m_ = game.undo_move();
            // Specifying >= here would let me look at less positions. But I can no longer trust an equal score. If the scores are equal I need to take the first.
            // But I want the engine to take a random move among the best - so I need to be able to trust ties.
            if score > b {
                if depth >= 0 && m_.eaten_loc == 0 {
                    *killer_move_cache.entry(game.get_game_len()).or_insert(HashMap::with_hasher(A {})).entry(m_.hash()).or_insert(0) += 1;
                }
                break;
            }
            a = max(a, score);
        }
    } else {
        score = Chess::MAX_INFINITY - (game.get_game_len() * 100) as <Chess as Scored>::ScoreType;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            let move_score = if depth > 1 || to != 0 {alpha_beta(game, depth - 1, a, b, to, killer_move_cache, call_count, max_timestamp_ms)?} else { game.get_score() };
            score = min(score, move_score);
            let m_ = game.undo_move();
            if score < a {
                if depth >= 0 && m_.eaten_loc == 0 {
                    *killer_move_cache.entry(game.get_game_len()).or_insert(HashMap::with_hasher(A {})).entry(m_.hash()).or_insert(0) += 1;
                }
                break;
            }
            b = min(b, score);
        }
    }
    return Some(score);
}

