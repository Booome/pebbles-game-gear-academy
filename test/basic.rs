#[cfg(test)]
mod tests {
    use gtest::{constants, Program, System};
    use pebbles_game_io::{
        DifficultyLevel, GameState, PebblesAction, PebblesEvent, PebblesInit, Player,
    };

    const USER_ID: u64 = constants::DEFAULT_USER_ALICE;

    #[test]
    fn test_init_user_first() {
        let sys = System::new();
        let prog = Program::current(&sys);
        sys.mint_to(USER_ID, constants::EXISTENTIAL_DEPOSIT * 1000);

        let pebbles_init = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 10,
            mock_rng: Some(vec![0u32]),
        };
        let message_id = prog.send(USER_ID, pebbles_init);
        let block_run_result = sys.run_next_block();
        assert!(block_run_result.succeed.contains(&message_id));

        let game_state: GameState = prog.read_state(()).unwrap();
        assert!(game_state.pebbles_count == 100);
        assert!(game_state.max_pebbles_per_turn == 10);
        assert!(game_state.pebbles_remaining == game_state.pebbles_count);
        assert!(game_state.difficulty == DifficultyLevel::Easy);
        assert!(game_state.first_player == Player::User);
        assert!(game_state.winner.is_none());
    }

    #[test]
    fn test_init_program_first() {
        let sys = System::new();
        let prog = Program::current(&sys);
        sys.mint_to(USER_ID, constants::EXISTENTIAL_DEPOSIT * 1000);

        let pebbles_init = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 10,
            mock_rng: Some(vec![1u32, 2u32]),
        };
        let message_id = prog.send(USER_ID, pebbles_init);
        let block_run_result = sys.run_next_block();
        assert!(block_run_result.succeed.contains(&message_id));

        let game_state: GameState = prog.read_state(()).unwrap();
        assert!(game_state.pebbles_count == 100);
        assert!(game_state.max_pebbles_per_turn == 10);
        assert!(game_state.pebbles_remaining == 97);
        assert!(game_state.difficulty == DifficultyLevel::Easy);
        assert!(game_state.first_player == Player::Program);
        assert!(game_state.winner.is_none());
    }

    #[test]
    fn test_init_program_win_after_first_turn() {
        let sys = System::new();
        let prog = Program::current(&sys);
        sys.mint_to(USER_ID, constants::EXISTENTIAL_DEPOSIT * 1000);

        let pebbles_init = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 9,
            max_pebbles_per_turn: 10,
            mock_rng: Some(vec![1u32, 2u32]),
        };
        let message_id = prog.send(USER_ID, pebbles_init);
        let block_run_result = sys.run_next_block();
        assert!(block_run_result.succeed.contains(&message_id));

        let log = block_run_result.decoded_log::<PebblesEvent>();
        let event = log.get(0).unwrap().payload();
        assert!(matches!(event, PebblesEvent::Won(Player::Program)));

        let game_state: GameState = prog.read_state(()).unwrap();
        assert!(game_state.pebbles_remaining == 0);
        assert!(game_state.winner == Some(Player::Program));

        let pebbles_action = PebblesAction::Turn(1);
        let message_id = prog.send(USER_ID, pebbles_action);
        let block_run_result = sys.run_next_block();
        block_run_result.assert_panicked_with(message_id, "Game is over");
    }

    #[test]
    fn test_init_invalid_pebbles_count() {
        let sys = System::new();
        let prog = Program::current(&sys);
        sys.mint_to(USER_ID, constants::EXISTENTIAL_DEPOSIT * 1000);

        let pebbles_init = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 0,
            max_pebbles_per_turn: 10,
            mock_rng: Some(vec![1u32, 2u32]),
        };
        let message_id = prog.send(USER_ID, pebbles_init);
        let block_run_result = sys.run_next_block();
        block_run_result.assert_panicked_with(message_id, "Invalid game initialization");
    }

    #[test]
    fn test_init_invalid_max_pebbles_per_turn() {
        let sys = System::new();
        let prog = Program::current(&sys);
        sys.mint_to(USER_ID, constants::EXISTENTIAL_DEPOSIT * 1000);

        let pebbles_init = PebblesInit {
            difficulty: DifficultyLevel::Easy,
            pebbles_count: 100,
            max_pebbles_per_turn: 0,
            mock_rng: Some(vec![1u32, 2u32]),
        };
        let message_id = prog.send(USER_ID, pebbles_init);
        let block_run_result = sys.run_next_block();
        block_run_result.assert_panicked_with(message_id, "Invalid game initialization");
    }
}
