use crate::two_player_game::{Game, Scored};
use crate::alpha_beta::get_next_move;
use crate::two_player_game::GameState::PLAYING;

mod two_player_game;
mod alpha_beta;
mod xo_impl;



fn main() {
    let mut xo = xo_impl::Xo::new_game();
    xo.console_draw();

    while xo.game_state() == PLAYING {
        let m = get_next_move(&mut xo);
        //println!("{:?}", m);
        xo.do_move(&m);
        println!();
        xo.console_draw();
        //println!("{:?}", xo.get_score());


    }
}
