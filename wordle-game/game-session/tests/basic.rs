#![cfg(test)]
use super::*;
use gstd::collections::HashMap;
use gstd::prelude::*;
use gstd::ActorId;
use gtest::{Program, System};
use io::*;

// 测试初始化函数
#[test]
fn test_init() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);

    let result = program.send(1, wordle_program);
    assert!(result.contains("Successfully initialized"), "Initialization failed");
}

// 测试开始游戏
#[test]
fn test_start_game() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);
    program.send(1, wordle_program);

    let user = ActorId::new([2u8; 32]);
    let result = program.send(2, Action::StartGame { user });
    assert!(result.contains("Game successfully started"), "Failed to start game");

    let state = program.send(2, Action::CheckGameStatus { user });
    assert!(state.contains("Game is in progress"), "Game status is not as expected");
}

// 测试猜词功能
#[test]
fn test_check_word() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);
    program.send(1, wordle_program);

    let user = ActorId::new([2u8; 32]);
    program.send(2, Action::StartGame { user });

    let result = program.send(2, Action::CheckWord {
        user,
        word: "apple".to_string(),
    });
    assert!(result.contains("Word checked successfully"), "Failed to check word");
}

// 测试结束游戏
#[test]
fn test_end_game() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);
    program.send(1, wordle_program);

    let user = ActorId::new([2u8; 32]);
    program.send(2, Action::StartGame { user });

    sys.spend_blocks(200); // 模拟时间经过

    let result = program.send(2, Action::EndGame { user });
    assert!(result.contains("Game over due to timeout"), "Failed to end game");
}

// 测试游戏状态查询
#[test]
fn test_game_status_query() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);
    program.send(1, wordle_program);

    let user = ActorId::new([2u8; 32]);
    program.send(2, Action::StartGame { user });

    let state = program.send(2, Action::CheckGameStatus { user });
    assert!(state.contains("Game is in progress"), "Game status is not as expected");
}

// 测试处理回复消息
#[test]
fn test_handle_reply() {
    let sys = System::new();
    sys.init_logger();

    let program = Program::current(&sys);
    let wordle_program = ActorId::new([1u8; 32]);
    program.send(1, wordle_program);

    let user = ActorId::new([2u8; 32]);
    program.send(2, Action::StartGame { user });

    let reply_event = Event::GameStarted { user };
    let result = program.send(1, reply_event);
    assert!(result.contains("Reply handled successfully"), "Failed to handle reply");
}
