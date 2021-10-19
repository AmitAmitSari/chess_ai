use crate::two_player_game::Game;
use crate::two_player_game::Scored;
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use std::cmp::{max, min};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use crate::chess_impl::Chess;

pub fn get_next_move(game: &mut Chess, depth: i32) -> Option<<Chess as Game>::MoveType>
{
    let mut rng = rand::thread_rng();
    let mut best_moves = vec![];
    let mut score;
    let mut a = Chess::MIN_INFINITY;
    let mut b = Chess::MAX_INFINITY;

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY;
        for m in game.possible_moves() {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to);
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
        for m in game.possible_moves() {
            let to = m.to;
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b, to);
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
    eprintln!("Expected score: {}", score);
    return m;
}

fn alpha_beta(game: &mut Chess, depth: i32, mut a: <Chess as Scored>::ScoreType, mut b: <Chess as Scored>::ScoreType, last_to: u64) -> <Chess as Scored>::ScoreType
{
    let mut possible_moves = game.possible_moves();

    if depth <= 0 {
        // Quiescence.
        possible_moves.retain(|m| m.eaten_loc == last_to);
        if possible_moves.len() == 0 {
            return game.get_score();
        }
    }

    let mut score;

    if game.current_player() == PLAYER1 {
        score = Chess::MIN_INFINITY;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            score = max(score, alpha_beta(game, depth - 1, a, b, to));
            game.undo_move();
            // Specifying >= here would let me look at less positions. But I can no longer trust an equal score. If the scores are equal I need to take the first.
            // But I want the engine to take a random move among the best - so I need to be able to trust ties.
            if score > b {
                break;
            }
            a = max(a, score);
        }
    } else {
        score = Chess::MAX_INFINITY;
        for m in possible_moves {
            let to = m.to;
            game.do_move(m);
            score = min(score, alpha_beta(game, depth - 1, a, b, to));
            game.undo_move();
            if score < a {
                break;
            }
            b = min(b, score);
        }
    }
    return score;
}

