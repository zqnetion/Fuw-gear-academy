use gtest::{Log, Program, System};
use pebbles_game_io::*;

const PLAYER: u64 = 100;

fn init_game(sys: &System, difficulty: DifficultyLevel, pebbles_count: u32, max_pebbles_per_turn: u32) -> Program<'_> {
    sys.init_logger();
    let game = Program::current_opt(sys);

    let pebbles_init = PebblesInit {
        difficulty: difficulty,
        pebbles_count: pebbles_count,
        max_pebbles_per_turn: max_pebbles_per_turn,
    };
    let res = game.send(PLAYER, pebbles_init);
    assert!(!res.main_failed());
    game
}

#[test]
fn init_ok() {
    let sys: System = System::new();
    sys.init_logger();
    let game = Program::current_opt(&sys);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 10,
        max_pebbles_per_turn: 5,
    };
    let res = game.send(PLAYER, pebbles_init);
    assert!(!res.main_failed());

    let expected = Log::builder().payload(PebblesEvent::CounterTurn(4));
    assert!(res.contains(&expected));
}

#[test]
fn init_failed() {
    let sys: System = System::new();
    sys.init_logger();
    let game = Program::current_opt(&sys);

    let pebbles_init = PebblesInit {
        difficulty: DifficultyLevel::Easy,
        pebbles_count: 0,
        max_pebbles_per_turn: 0,
    };
    let res = game.send(PLAYER, pebbles_init);
    assert!(res.main_failed());
}

#[test]
fn init_with_easy_level() {
    let sys: System = System::new();
    let game = init_game(&sys, DifficultyLevel::Easy, 10, 5);

    let state: GameState = game.read_state(b"").unwrap();
    assert_eq!(state.pebbles_count, 10);
    assert_eq!(state.max_pebbles_per_turn, 5);

    // In easy mode, Program `randomly` removes 2 peddles, and there are 8 peddles left.
    assert_eq!(state.pebbles_remaining, 8);

    assert!(matches!(state.first_player, Player::Program));
    assert!(matches!(state.difficulty, DifficultyLevel::Easy));
}

#[test]
fn init_with_hard_level() {
    let sys: System = System::new();
    let game = init_game(&sys, DifficultyLevel::Hard, 10, 5);

    let state: GameState = game.read_state(b"").unwrap();
    assert_eq!(state.pebbles_count, 10);
    assert_eq!(state.max_pebbles_per_turn, 5);

    // In hard mode, Program chooses the best choice and removes 4 peddles, and there are 6 peddles left.
    assert_eq!(state.pebbles_remaining, 6);
    assert_eq!(state.pebbles_remaining % (state.max_pebbles_per_turn + 1), 0);

    assert!(matches!(state.first_player, Player::Program));
    assert!(matches!(state.difficulty, DifficultyLevel::Hard));
}

#[test]
fn user_win() {
    // count % (max + 1) == 0, first player (Program) loses
    let sys: System = System::new();

    // Program `randomly` removes 1~4 peddles during `init`.
    let game = init_game(&sys, DifficultyLevel::Hard, 10, 4);

    loop {
        let state: GameState = game.read_state(b"").unwrap();
        if !state.winner.is_none() {
            let _expected_winner = Some(Player::User);
            assert!(matches!(state.winner, _expected_winner));
            break;
        }

        let user_pebbles = state.pebbles_remaining % (state.max_pebbles_per_turn + 1);
        assert!(!game.send(PLAYER, PebblesAction::Turn(user_pebbles)).main_failed());
    }
}

#[test]
fn program_win() {
    // count % (max + 1) != 0, first player (Program) wins
    let sys: System = System::new();

    // Program removes 4 peddles during `init`, and there are 6 peddles left.
    let game = init_game(&sys, DifficultyLevel::Hard, 10, 5);

    // User removes 3 peddles, and there are 3 peddles left.
    // Program removes the remaining 3 peddles and wins the game.
    assert!(!game.send(PLAYER, PebblesAction::Turn(3)).main_failed());

    let state: GameState = game.read_state(b"").unwrap();
    let _expected_winner = Some(Player::Program);
    assert!(matches!(state.winner, _expected_winner));
}

#[test]
fn user_give_up() {
    let sys: System = System::new();
    let game = init_game(&sys, DifficultyLevel::Easy, 10, 5);
    assert!(!game.send(PLAYER, PebblesAction::GiveUp).main_failed());

    let state: GameState = game.read_state(b"").unwrap();
    let _expected_winner = Some(Player::Program);
    assert!(matches!(state.winner, _expected_winner));
}

#[test]
fn restart() {
    let sys: System = System::new();
    let game = init_game(&sys, DifficultyLevel::Easy, 10, 5);
    let state1: GameState = game.read_state(b"").unwrap();
    assert_eq!(state1.pebbles_count, 10);
    assert_eq!(state1.max_pebbles_per_turn, 5);
    assert_eq!(state1.pebbles_remaining,8);
    assert!(matches!(state1.difficulty, DifficultyLevel::Easy));

    assert!(!game.send(PLAYER, PebblesAction::Restart{
        difficulty: DifficultyLevel::Hard,
        pebbles_count: 20,
        max_pebbles_per_turn: 6,
    }).main_failed());

    let state2: GameState = game.read_state(b"").unwrap();
    assert_eq!(state2.pebbles_count, 20);
    assert_eq!(state2.max_pebbles_per_turn, 6);
    assert_eq!(state2.pebbles_remaining, 14);
    assert!(matches!(state2.difficulty, DifficultyLevel::Hard));
}
