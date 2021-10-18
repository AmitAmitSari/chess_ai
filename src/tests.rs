

#[cfg(test)]
mod tests {
    use crate::chess_impl::Chess;
    use crate::two_player_game::Game;

    fn count_positions(chess: &mut Chess, depth: i32) -> usize {
        if depth == 1 {
            return chess.possible_moves().len();
        }

        let moves = chess.possible_moves();
        let mut res = 0;
        for m in moves {
            chess.do_move(m);
            res += count_positions(chess, depth - 1);
            chess.undo_move();
        }
        res
    }

    #[test]
    fn test_position() {
        let mut chess = Chess::new();
        chess.setup_fen_string("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3");

        chess.console_draw();

        for m in chess.possible_moves() {
            println!("Move: {}", m)
        }

    }

    #[test]
    fn test_stuff() {
        println!("starting");

        let cases = [
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 1, 20),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2, 400),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3, 8902),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 4, 197281),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4865609),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),

            // Black check by bishop
            ("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2", 1, 8),
            //
            ("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3", 1, 8),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
            // ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
        ];


        for (fen, depth, expected) in cases.iter().copied() {
            let mut chess = Chess::new();
            chess.setup_fen_string(fen);
            let actual = count_positions(&mut chess, depth);

            println!("Checking, {}, {}, {}, {}", depth, expected, actual, fen);
            assert_eq!(expected, actual);
        }

    }

}
