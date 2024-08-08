#![no_std]
use gstd::{msg, exec, prelude::*, ActorId, debug, collections::HashMap,msg::send_delayed};
use game_session_io::*;

static mut GAME_SESSION: Option<GameSession> = None;


#[no_mangle]
extern "C" fn init() {
    let wordle_program: ActorId = match msg::load() {
        Ok(actor_id) => actor_id,
        Err(_) => {
            debug!("Failed to load ActorId");
            return;
        }
    };
    unsafe {
        GAME_SESSION = Some(GameSession {
            wordle_program,
            games: HashMap::new(),
        });
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: Action = match msg::load() {
        Ok(action) => action,
        Err(_) => {
            debug!("Failed to load action");
            msg::reply("Failed to load action", 0).expect("Failed to send reply");
            return;
        }
    };

    let game_session = unsafe {
        GAME_SESSION.as_mut().expect("Game session is not initialized")
    };

    match action {
        Action::EndGame { user } => {
            let game_status = match game_session.games.get_mut(&user) {
                Some(status) => status,
                None => {
                    msg::reply("Game does not exist for the user", 0).expect("Failed to send reply");
                    return;
                }
            };
            if game_status.status == GameState::InProgress {
                game_status.status = GameState::GameOver(Outcome::Lose);
                msg::reply("Game over due to timeout", 0).expect("Failed to send reply");
            }
        }
        Action::StartGame { user } => {
            if game_session.games.contains_key(&user) {
                msg::reply("Game already exists for the user", 0).expect("Failed to send reply");
                return;
            }

            let start_game_msg = Action::StartGame { user };
            msg::send(game_session.wordle_program, start_game_msg.encode(), 0)
                .expect("Failed to send StartGame message");

            game_session.games.insert(user, GameStatus {
                word: None,
                attempts: 0,
                status: GameState::InProgress,
            });

            // 添加延迟消息
            exec::send_delayed(
                exec::program_id(),
                Action::EndGame { user }.encode(),
                200,
            ).expect("Failed to send delayed message");

            msg::reply("Game successfully started", 0).expect("Failed to send reply");
            exec::wait();
        }
        Action::CheckWord { user, word } => {
            let game_status = match game_session.games.get_mut(&user) {
                Some(status) => status,
                None => {
                    msg::reply("Game does not exist for the user", 0).expect("Failed to send reply");
                    return;
                }
            };

            if game_status.status != GameState::InProgress {
                msg::reply("Game is not in progress", 0).expect("Failed to send reply");
                return;
            }

            if word.len() != 5 || !word.chars().all(char::is_lowercase) {
                msg::reply("Invalid word format", 0).expect("Failed to send reply");
                return;
            }

            let check_word_msg = Action::CheckWord { user, word: word.clone() };
            msg::send(game_session.wordle_program, check_word_msg.encode(), 0)
                .expect("Failed to send CheckWord message");
            game_status.attempts += 1;
            msg::reply("Word checked successfully", 0).expect("Failed to send reply");
            exec::wait();
        }
        Action::CheckGameStatus { user } => {
            let game_status = match game_session.games.get(&user) {
                Some(status) => status,
                None => {
                    msg::reply("Game does not exist for the user", 0).expect("Failed to send reply");
                    return;
                }
            };

            let status_message = match game_status.status {
                GameState::NotStarted => "Game not started",
                GameState::InProgress => "Game is in progress",
                GameState::GameOver(Outcome::Win) => "Game over: Win",
                GameState::GameOver(Outcome::Lose) => "Game over: Lose",
            };

            msg::reply(status_message, 0).expect("Failed to send reply");
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply: Event = match msg::load() {
        Ok(reply) => reply,
        Err(_) => {
            debug!("Failed to load event");
            msg::reply("Failed to load event", 0).expect("Failed to send reply");
            return;
        }
    };

    let game_session = unsafe {
        GAME_SESSION.as_mut().expect("Game session is not initialized")
    };

    match reply {
        Event::GameStarted { user } => {
            let game_status = match game_session.games.get_mut(&user) {
                Some(status) => status,
                None => {
                    msg::reply("Game does not exist for the user", 0).expect("Failed to send reply");
                    return;
                }
            };
            game_status.status = GameState::InProgress;
            debug!("Game started for user: {:?}", user);
        }
        Event::WordChecked { user, correct_positions, contained_in_word } => {
            let game_status = match game_session.games.get_mut(&user) {
                Some(status) => status,
                None => {
                    msg::reply("Game does not exist for the user", 0).expect("Failed to send reply");
                    return;
                }
            };

            if correct_positions.len() == 5 {
                game_status.status = GameState::GameOver(Outcome::Win);
                debug!("User {:?} has guessed the word correctly!", user);
            } else if game_status.attempts >= 6 {
                game_status.status = GameState::GameOver(Outcome::Lose);
                debug!("User {:?} has used all attempts and failed to guess the word.", user);
            }

            debug!(
                "Word checked for user: {:?}, correct_positions: {:?}, contained_in_word: {:?}",
                user, correct_positions, contained_in_word
            );
        }
    }

    msg::reply("Reply handled successfully", 0).expect("Failed to send reply");
}

#[no_mangle]
extern "C" fn state() {
    let game_session = unsafe {
        GAME_SESSION.as_ref().expect("Game session is not initialized")
    };
    let state_info = game_session.games.iter().map(|(user, status)| {
        (
            user,
            match status.status {
                GameState::NotStarted => "NotStarted",
                GameState::InProgress => "InProgress",
                GameState::GameOver(Outcome::Win) => "GameOver(Win)",
                GameState::GameOver(Outcome::Lose) => "GameOver(Lose)",
            },
            status.attempts,
            status.word.as_deref().unwrap_or(""),
        )
    }).collect::<Vec<_>>();

    msg::reply(state_info, 0).expect("Failed to share state");
}
