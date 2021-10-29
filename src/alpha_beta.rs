use crate::two_player_game::Game;
use crate::two_player_game::Scored;
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use std::cmp::{max, min};
use std::collections::{HashMap, HashSet};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use crate::chess_impl::{Chess, Move};

type Cache = HashMap<i32, HashMap<Move, i32>>;

pub fn get_next_move(game: &mut Chess, depth: i32) -> Option<<Chess as Game>::MoveType> {

    let mut killer_move_cache: Cache = HashMap::new();
    let mut m = None;
    let mut total_count = 0;
    for i in min(depth-2,depth)..depth+1 {
        let call_count: &mut i32 = &mut 0;
        m = _get_next_move(game, i, &mut killer_move_cache, call_count);
        total_count += *call_count;
        println!("Depth: {}, CallCount: {}, Total: {}", i, call_count, total_count);
    }
    // let mut cache = killer_move_cache.iter().collect::<Vec<_>>();
    // cache.sort_unstable_by_key(|(k, v)| *k );
    // for (key, value) in cache {
    //
    //     let mut moves = value.iter().collect::<Vec<_>>();
    //     moves.sort_unstable_by_key(|(_,&v)| -v);
    //     let moves = moves.iter().map(|(m, c)| m.to_string() + ": " + &c.to_string()).collect::<Vec<_>>();
    //     println!("{}: {:?}", key, moves);
    // }
    m
}

#[inline]
fn move_ordering(m: &Move, depth: i32, killer_move_cache: &Cache) -> i32 {
    -*killer_move_cache.get(&depth).and_then(|d| d.get(m)).unwrap_or(&((m.eaten_loc != 0) as i32 * 10))
}

fn _get_next_move(game: &mut Chess, depth: i32, mut killer_move_cache: &mut Cache, call_count: &mut i32) -> Option<<Chess as Game>::MoveType>
{
    let mut rng = rand::thread_rng();
    let mut best_moves = vec![];
    let mut score;
    let mut a = Chess::MIN_INFINITY;
    let mut b = Chess::MAX_INFINITY;

    let mut possible_moves = game.possible_moves();
    *call_count += 1;
    possible_moves.sort_by_cached_key(|m| move_ordering(m, depth, killer_move_cache));

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to, &mut killer_move_cache, call_count);
            let m_ = game.undo_move();
            if move_score >= score {
                if move_score > score {
                    // println!("{}, {}", m_, move_score);
                    best_moves.clear();
                }
                best_moves.push(m_);
                score = move_score
            }
            a = max(a, score);
        }
    } else {
        score = Chess::MAX_INFINITY;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to, &mut killer_move_cache, call_count);
            let m_ = game.undo_move();
            if move_score <= score {
                if move_score < score {
                    // println!("{}, {}", m_, move_score);
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
    eprintln!("Expected score: {}, choose from {} other moves", score, best_moves.len());
    if let Some(m_) = m.clone() {
        *killer_move_cache.entry(depth).or_insert(HashMap::new()).entry(m_).or_insert(0) += 1;
    }
    return m;
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

pub fn alpha_beta(game: &mut Chess, depth: i32, mut a: <Chess as Scored>::ScoreType, mut b: <Chess as Scored>::ScoreType, last_to: u64, killer_move_cache: &mut Cache, call_count: &mut i32) -> <Chess as Scored>::ScoreType
{
    let mut possible_moves = game.possible_moves();
    *call_count += 1;

    if depth <= 0 {
        // Quiescence.
        possible_moves.retain(|m| m.eaten_loc != 0 && m.eaten_loc == last_to);
        if possible_moves.len() == 0 {
            return game.get_score();
        }
    } else {
        possible_moves.sort_by_cached_key(|m| move_ordering(m, depth, killer_move_cache));
    }

    let mut score;

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            let move_score = if depth > 1 || to != 0 {alpha_beta(game, depth - 1, a, b, to, killer_move_cache, call_count)} else { game.get_score() };
            score = max(score, move_score);
            let m_ = game.undo_move();
            // Specifying >= here would let me look at less positions. But I can no longer trust an equal score. If the scores are equal I need to take the first.
            // But I want the engine to take a random move among the best - so I need to be able to trust ties.
            if score > b {
                if depth >= 0 && m_.eaten_loc == 0 {
                    *killer_move_cache.entry(depth).or_insert(HashMap::new()).entry(m_).or_insert(0) += 1;
                }
                break;
            }
            a = max(a, score);
        }
    } else {
        score = Chess::MAX_INFINITY;
        for m in possible_moves {
            let to = m.eaten_loc;
            game.do_move(m);
            let move_score = if depth > 1 || to != 0 {alpha_beta(game, depth - 1, a, b, to, killer_move_cache, call_count)} else { game.get_score() };
            score = min(score, move_score);
            let m_ = game.undo_move();
            if score < a {
                if depth >= 0 && m_.eaten_loc == 0 {
                    *killer_move_cache.entry(depth).or_insert(HashMap::new()).entry(m_).or_insert(0) += 1;
                }
                break;
            }
            b = min(b, score);
        }
    }
    return score;
}

