

#[cfg(test)]
mod tests {
    use crate::chess_impl::Chess;
    use crate::two_player_game::Game;

    fn count_positions(chess: &mut Chess, depth: i32) -> usize {
        if depth == 0 {
            return 1;
        }
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
        chess.setup_fen_string("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1");

        chess.console_draw();
        let depth = 1;
        for i in 1..depth+1 {
            println!("{}", count_positions(&mut chess, i));
        }

        for m in chess.possible_moves() {
            let sm = m.to_string();
            chess.do_move(m);
            let fen = chess.get_fen_string();
            let cnt = count_positions(&mut chess, depth - 1);
            let m_ = chess.undo_move();
            println!("Move {} {} '{}'", m_, cnt, fen)
        }

    }

    #[test]
    fn test_stuff() {
        println!("starting");

        let cases = [
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 1, 20),

            // Black check by bishop
            ("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2", 1, 8),
            // check and break by en passant capture
            ("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3", 1, 8),
            // ??
            ("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2", 1, 19),
            // Test no castle in check
            ("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2", 1, 5),
            // Test castle
            ("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9", 1, 39),

            // ??
            ("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1, 9),
            ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 1, 44),
            ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 2, 1486),
            ("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3, 62379),
            ("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 3, 89890),
            ("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6, 1134888),
            ("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6, 1015133),
            ("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6, 1440467),
            ("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6, 661072),
            ("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6, 803711),
            ("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4, 1274206),
            ("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4, 1720476),
            ("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6, 3821001),
            ("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5, 1004658),
            ("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6, 217342),
            ("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6, 92683),
            ("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6, 2217),
            ("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7, 567584),
            ("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4, 23527),

            // Starting position
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2, 400),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3, 8902),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 4, 197281),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5, 4865609),
            ("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6, 119060324),
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
