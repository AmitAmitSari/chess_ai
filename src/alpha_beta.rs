use crate::two_player_game::Game;
use crate::two_player_game::Scored;
use crate::two_player_game::GameState::PLAYING;
use crate::two_player_game::Player::PLAYER1;
use std::cmp::{max, min};

pub fn get_next_move<Y>(game: &mut Y) -> Option<Y::MoveType>
    where Y: Game + Scored
{
    min_max(game, 100).0
}

fn min_max<Y>(game: &mut Y, depth: i32) -> (Option<<Y as Game>::MoveType>, Y::ScoreType)
    where Y: Game + Scored
{
    if depth == 0 || game.game_state() != PLAYING {
        return (None, game.get_score());
    }

    let mut score = if game.current_player() == PLAYER1 { Y::MIN_INFINITY } else { Y::MAX_INFINITY };
    let func = if game.current_player() == PLAYER1 { max } else { min };
    let mut best_move: Option<<Y as Game>::MoveType> = None;

    for m in game.possible_moves() {
        game.do_move(m);
        let move_score = min_max(game, depth - 1).1;
        let m_ = game.undo_move();

        if func(score, move_score) != score {
            best_move = Some(m_);
        }
        score = func(score, move_score);
    }

    return (best_move, score);
}

