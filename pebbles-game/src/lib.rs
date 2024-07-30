#![no_std]

use gstd::{msg, exec, debug};
use pebbles_game_io::*;

static mut PEBBLES_GAME: Option<GameState> = None;
#[cfg(test)]
fn get_random_u32() -> u32 {
    2
}

#[cfg(not(test))]
fn get_random_u32() -> u32 {
    let salt = msg::id();
    let (hash, _num) = exec::random(salt.into()).expect("get_random_u32(): random call failed");
    u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
}

fn random_first_player() -> Player {
    if get_random_u32() % 2 == 0 {
        Player::Program
    } else {
        Player::User
    }
}

fn get_program_pebbles(game_state: &GameState) -> u32 {
    match game_state.difficulty {
        DifficultyLevel::Easy => get_random_u32() % game_state.max_pebbles_per_turn + 1,
        DifficultyLevel::Hard => {
            let remainder = game_state.pebbles_remaining % (game_state.max_pebbles_per_turn + 1);
            if remainder != 0 {
                remainder
            } else {
                get_random_u32() % game_state.max_pebbles_per_turn + 1
            }
        },
    }
}

fn do_turn(game_state: &mut GameState, pebbles: u32, player: Player) {
    if pebbles >= game_state.pebbles_remaining {
        game_state.pebbles_remaining = 0;
        game_state.winner = Some(player);
    } else {
        game_state.pebbles_remaining -= pebbles;
    }
}

fn program_turn(game_state: &mut GameState) {
    let pebbles = get_program_pebbles(game_state);
    debug!("remaining pebbles: {}, program remove pebbles: {}", game_state.pebbles_remaining, pebbles);

    do_turn(game_state, pebbles, Player::Program);

    if game_state.winner.is_none() {
        msg::reply(PebblesEvent::CounterTurn(pebbles), 0).expect("Unable to reply");
    } else {
        msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
    }
}

fn user_turn(game_state: &mut GameState, pebbles: u32) {
    debug!("remaining pebbles: {}, user remove pebbles: {}", game_state.pebbles_remaining, pebbles);
    if pebbles > game_state.max_pebbles_per_turn {
        panic!("pebbles > max_pebbles_per_turn");
    }

    if !game_state.winner.is_none() {
        panic!("Game already finished");
    }

    do_turn(game_state, pebbles, Player::User);
    if !game_state.winner.is_none() {
        msg::reply(PebblesEvent::Won(Player::User), 0).expect("Unable to reply");
    } else {
        program_turn(game_state);
    }
}

fn give_up(game_state: &mut GameState) {
    if !game_state.winner.is_none() {
        panic!("Game already finished");
    }

    game_state.winner = Some(Player::Program);
    msg::reply(PebblesEvent::Won(Player::Program), 0).expect("Unable to reply");
}

fn restart(game_state: &mut GameState, difficulty: DifficultyLevel, pebbles_count: u32, max_pebbles_per_turn: u32) {
    game_state.pebbles_count = pebbles_count;
    game_state.max_pebbles_per_turn = max_pebbles_per_turn;
    game_state.pebbles_remaining = pebbles_count;
    game_state.difficulty = difficulty;
    game_state.first_player = random_first_player();
    game_state.winner = None;

    if let Player::Program = game_state.first_player {
        program_turn(game_state);
    }
}

#[no_mangle]
extern "C" fn init() {
    let init_params: PebblesInit = msg::load().expect("Failed to load PebblesInit");
    debug!("init payload: {:x?}", &init_params);

    if init_params.pebbles_count == 0 || init_params.max_pebbles_per_turn == 0 {
        panic!("Invalid pebbles_count or max_pebbles_per_turn");
    }

    let mut game_state = GameState {
        pebbles_count: init_params.pebbles_count,
        max_pebbles_per_turn: init_params.max_pebbles_per_turn,
        pebbles_remaining: init_params.pebbles_count,
        difficulty: init_params.difficulty,
        first_player: random_first_player(),
        winner: None,
    };

    if let Player::Program = game_state.first_player {
        program_turn(&mut game_state);
    }

    unsafe {
        PEBBLES_GAME = Some(game_state);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: PebblesAction = msg::load().expect("Failed to load PebblesAction");
    debug!("handle payload: {:x?}", &action);

    let game_state = unsafe { PEBBLES_GAME.as_mut().expect("`GAME_STATE` is not initialized")};
    match action {
        PebblesAction::Turn(user_pebbles) => user_turn(game_state, user_pebbles),
        PebblesAction::GiveUp => give_up(game_state),
        PebblesAction::Restart { difficulty, pebbles_count, max_pebbles_per_turn } =>
            restart(game_state, difficulty, pebbles_count, max_pebbles_per_turn),
    }
}

#[no_mangle]
extern "C" fn state() {
    let game_state = unsafe { PEBBLES_GAME.take().expect("Unexpected error in taking state") };
    msg::reply::<GameState>(game_state.into(), 0)
        .expect("Failed to encode or reply with `GameState` from `state()`");
}
