use crate::two_player_game::Game;
use crate::two_player_game::Scored;
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use std::cmp::{max, min};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};

pub fn get_next_move<Y>(game: &mut Y, depth: i32) -> Option<Y::MoveType>
    where Y: Game + Scored
{
    let mut rng = rand::thread_rng();
    let mut best_moves = vec![];
    let mut score;
    let mut a = Y::MIN_INFINITY;
    let mut b = Y::MAX_INFINITY;

    if game.current_player() == PLAYER1 {
        score = Y::MIN_INFINITY;
        for m in game.possible_moves() {
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b);
            let m_ = game.undo_move();
            if move_score >= score {
                if move_score > score {
                    println!("{}, {}", m_, move_score);
                    best_moves.clear();
                }
                best_moves.push(m_);
                score = move_score
            }
            a = max(a, score);
        }
    } else {
        score = Y::MAX_INFINITY;
        for m in game.possible_moves() {
            game.do_move(m);
            let move_score = alpha_beta(game, depth - 1, a, b);
            let m_ = game.undo_move();
            if move_score <= score {
                if move_score < score {
                    println!("{}, {}", m_, move_score);
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

    return m;
}

fn alpha_beta<Y>(game: &mut Y, depth: i32, mut a: Y::ScoreType, mut b: Y::ScoreType) -> Y::ScoreType
    where Y: Game + Scored
{
    if depth == 0 {
        return game.get_score();
    }

    let mut score;

    if game.current_player() == PLAYER1 {
        score = Y::MIN_INFINITY;
        for m in game.possible_moves() {
            game.do_move(m);
            score = max(score, alpha_beta(game, depth - 1, a, b));
            game.undo_move();
            if score >= b {
                break;
            }
            a = max(a, score);
        }
    } else {
        score = Y::MAX_INFINITY;
        for m in game.possible_moves() {
            game.do_move(m);
            score = min(score, alpha_beta(game, depth - 1, a, b));
            game.undo_move();
            if score <= a {
                break;
            }
            b = min(b, score);
        }
    }
    return score;
}

