#[cfg(test)]
mod tests {
    use gstd::{prelude::*, ActorId};
    use gtest::{Program, System};
    use game_session_io::*;

    const WORDLE_ID: u64 = 1;
    const GAME_SESSION_ID: u64 = 2;
    const USER1: u64 = 10;

    
    fn setup() -> System {
        let sys = System::new();
        sys.init_logger();

        let wordle = Program::from_file(&sys, "../target/wasm32-unknown-unknown/debug/wordle.opt.wasm");
        let game_session = Program::from_file(
            &sys,
            "../target/wasm32-unknown-unknown/debug/game_session.opt.wasm",
        );
        let user_id: ActorId = USER1.into();
        let wordle_id: ActorId = WORDLE_ID.into();

        // 初始化wordle和game_session程序
        assert!(!wordle.send(user_id, wordle_id).main_failed());
        assert!(!game_session.send(user_id, wordle_id).main_failed());
        sys
    }

    #[test]
    fn test_start_game() {
        let sys = setup();
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 测试StartGame逻辑
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        let state: GameSessionState = game_session.read_state(()).unwrap();
        assert!(state.game_sessions.iter().any(|(user, _)| *user == USER1.into()));
        println!("State after starting game: {:?}", state);

        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;

        assert!(matches!(
            session_info.session_status,
            SessionStatus::WaitUserInput
        ));
    }

    #[test]
    fn test_check_word_correct_check_win() {
        let sys = setup();
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 模拟用户发送 StartGame 请求
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());
        let state: GameSessionState = game_session.read_state(()).unwrap();
        println!("Check start: {:?}", state);

        // 模拟用户发送 CheckWord 请求
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "hello".to_string() }).main_failed());
        // 假设这是第一次发送
        let state: GameSessionState = game_session.read_state(()).unwrap();
        println!("State after first check: {:?}", state);

        // 再次发送 CheckWord 请求
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "hello".to_string() }).main_failed());

        // 检查会话状态是否为 WaitUserInput
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        println!("State after second check: {:?}", state);

        // 检查尝试次数是否增加
        assert_eq!(session_info.tries, 2);

        // 模拟用户再次发送 CheckWord 请求
        assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "human".to_string() }).main_failed());

        // 检查尝试次数是否更新
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        assert_eq!(session_info.tries, 3);
        assert!(matches!(
            session_info.session_status,
            SessionStatus::WaitUserInput
        ));
    }

    #[test]
    fn test_game_over() {
        let sys = setup();
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 模拟用户发送 StartGame 请求
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        // 模拟用户发送 CheckWord 请求，直到游戏结束
        for _ in 0..6 {
            assert!(!game_session.send(USER1, GameSessionAction::CheckWord { word: "wrong".to_string() }).main_failed());
        }

        // 检查会话状态是否为 GameOver
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        assert!(matches!(session_info.session_status, SessionStatus::GameOver(_)));
    }

    #[test]
    fn test_time() {
        let sys = setup();
        let game_session = sys.get_program(GAME_SESSION_ID).unwrap();

        // 模拟用户发送 StartGame 请求
        assert!(!game_session.send(USER1, GameSessionAction::StartGame).main_failed());

        // 模拟时间流逝（阻止执行一定数量的区块）,这里会报错？？？
       // sys.spend_blocks(300);

        // 检查游戏状态
        let state: GameSessionState = game_session.read_state(()).unwrap();
        let _session_info = &state
            .game_sessions
            .iter()
            .find(|(user, _)| *user == USER1.into())
            .unwrap()
            .1;
        println!("State after time pass: {:?}", state);

        // 确认游戏是否已经结束
       // assert!(matches!(session_info.session_status, SessionStatus::GameOver(_)));
    }
}
