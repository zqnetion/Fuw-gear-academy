#![no_std]
use game_session_io::*;
use gstd::{exec, msg, debug};

const TRIES_LIMIT: u8 = 6; // 每个游戏会话的最大尝试次数

static mut GAME_SESSION_STATE: Option<GameSession> = None;

#[no_mangle]
extern "C" fn init() {
    let game_session_init: GameSessionInit = msg::load().expect("Failed to decode GameSessionInit");
    game_session_init.assert_valid();

    unsafe {
        GAME_SESSION_STATE = Some(game_session_init.into());
    }
}

#[no_mangle]
extern "C" fn handle() {
    let game_session_action: GameSessionAction = msg::load().expect("Failed to decode GameSessionAction");
    let game_session = get_game_session_mut();
    let user = msg::source();
    match game_session_action {
        GameSessionAction::StartGame => {
            let session_info = game_session.sessions.entry(user).or_default();
            match session_info.session_status {
                SessionStatus::Init => {
                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::StartGame { user },
                        0,
                    ).expect("Failed to send message to Wordle");

                    session_info.session_status = SessionStatus::WaitWordleStartReply;
                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_id = msg::id(); // 更新会话 ID

                    msg::send_delayed(
                        exec::program_id(),
                        GameSessionAction::CheckGameStatus {
                            user,
                            session_id: session_info.session_id,
                        },
                        0,
                        200,
                    ).expect("Failed to send delayed message");
                    exec::wait();
                }
                _ => panic!("Invalid state for starting a game"),
            }
        }
        GameSessionAction::CheckWord { word } => {
            let session_info = game_session.sessions.entry(user).or_default();
            debug!("check{:?}", session_info);
            match session_info.session_status {
                SessionStatus::ReplyReceived(_) | SessionStatus::WaitUserInput => {
                    assert!(
                        word.len() == 5 && word.chars().all(|c| c.is_lowercase()),
                        "Invalid word"
                    );

                    let send_to_wordle_msg_id = msg::send(
                        game_session.wordle_program_id,
                        WordleAction::CheckWord { user, word },
                        0,
                    ).expect("Failed to send message to Wordle");

                    session_info.original_msg_id = msg::id();
                    session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
                    session_info.session_status = SessionStatus::WaitWordleCheckWordReply;

                    // Increment tries
                    session_info.tries += 1;
                    exec::wait();
                }
                _ => panic!("Invalid state for checking a word"),
            }
        }
        GameSessionAction::CheckGameStatus { user, session_id } => {
            if msg::source() == exec::program_id() {
                if let Some(session_info) = game_session.sessions.get_mut(&user) {
                    if session_id == session_info.session_id
                        && !matches!(session_info.session_status, SessionStatus::GameOver(..))
                    {
                        session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
                        msg::send(user, GameSessionEvent::GameOver(GameStatus::Lose), 0)
                            .expect("Failed to send GameOver message");
                    }
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn handle_reply() {
    let reply_to = msg::reply_to().expect("Failed to retrieve reply_to message");
    let wordle_event: WordleEvent = msg::load().expect("Failed to decode WordleEvent");
    let game_session = get_game_session_mut();
    let user = wordle_event.get_user();

    if let Some(session_info) = game_session.sessions.get_mut(user) {
        if reply_to == session_info.send_to_wordle_msg_id && session_info.is_wait_reply_status() {
            assert!(matches!(
                session_info.session_status,
                SessionStatus::WaitWordleStartReply | SessionStatus::WaitWordleCheckWordReply
            ), "Unexpected session status");

            // 更新状态为 ReplyReceived
            session_info.session_status = SessionStatus::ReplyReceived(wordle_event.clone());

            // 检查游戏是否胜利
            if wordle_event.has_guessed() {
                session_info.session_status = SessionStatus::GameOver(GameStatus::Win);
            }
            // 检查是否达到尝试次数限制
            else if session_info.tries >= TRIES_LIMIT {
                session_info.session_status = SessionStatus::GameOver(GameStatus::Lose);
            }
            // 否则继续等待用户输入
            else {
                session_info.session_status = SessionStatus::WaitUserInput;
            }

            // 唤醒等待中的消息
            exec::wake(reply_to).expect("Failed to wake up previous message");
        } else {
            panic!("Unexpected reply or session status does not match waiting state");
        }
    } else {
        panic!("Session info not found for user");
    }
}


#[no_mangle]
extern "C" fn state() {
    let game_session = get_game_session();
    let state: GameSessionState = game_session.into();
    msg::reply(state, 0).expect("Failed to send state reply");
}

fn get_game_session() -> &'static GameSession {
    unsafe { GAME_SESSION_STATE.as_ref().expect("Game session is not initialized") }
}

fn get_game_session_mut() -> &'static mut GameSession {
    unsafe { GAME_SESSION_STATE.as_mut().expect("Game session is not initialized") }
}
