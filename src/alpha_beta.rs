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
    min_max(game, depth, &mut rng).0
}

fn min_max<Y>(game: &mut Y, depth: i32, rng: &mut ThreadRng) -> (Option<<Y as Game>::MoveType>, Y::ScoreType)
    where Y: Game + Scored
{
    if depth == 0 || game.game_state() != PLAYING {
        return (None, game.get_score());
    }

    let mut score = if game.current_player() == PLAYER1 { Y::MIN_INFINITY } else { Y::MAX_INFINITY };
    let func = if game.current_player() == PLAYER1 { max } else { min };
    let mut best_moves: Vec<<Y as Game>::MoveType> = vec![];

    for m in game.possible_moves() {
        game.do_move(m);
        let move_score = min_max(game, depth - 1, rng).1;
        let m_ = game.undo_move();

        if func(score, move_score) != score {
            if depth == 4 {
                println!("{}, {}", m_, move_score)
            }
            if score != move_score {
                best_moves.clear();
                score = func(score, move_score);
            }
        }
        if score == move_score {
            best_moves.push(m_);
        }
    }

    let i = (0..best_moves.len()).choose(rng);

    let m = i.map(|x| best_moves.swap_remove(x));

    return (m, score);
}

