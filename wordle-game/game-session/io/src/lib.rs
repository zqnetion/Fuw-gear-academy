#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{collections::HashMap, prelude::*, msg, ActorId, MessageId, PartialEq};

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum SessionStatus {
    #[default]
    Init,
    WaitUserInput,
    WaitWordleStartReply,
    WaitWordleCheckWordReply,
    ReplyReceived(WordleEvent),
    GameOver(GameStatus),
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionState {
    pub wordle_program_id: ActorId,
    pub game_sessions: Vec<(ActorId, SessionInfo)>,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameSessionInit {
    pub wordle_program_id: ActorId,
}

impl GameSessionInit {
    pub fn assert_valid(&self) {
        assert!(
            !self.wordle_program_id.is_zero(),
            "Invalid wordle_program_id"
        );
    }
}

impl From<GameSessionInit> for GameSession {
    fn from(game_session_init: GameSessionInit) -> Self {
        Self {
            wordle_program_id: game_session_init.wordle_program_id,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionAction {
    StartGame,
    CheckWord {
        word: String,
    },
    CheckGameStatus {
        user: ActorId,
        session_id: MessageId,
    },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum WordleAction {
    StartGame { user: ActorId },
    CheckWord { user: ActorId, word: String },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum GameSessionEvent {
    StartSuccess,
    CheckWordResult {
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
    GameOver(GameStatus),
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum GameStatus {
    Win,
    Lose,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq)]
pub enum WordleEvent {
    GameStarted {
        user: ActorId,
    },
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
}

impl WordleEvent {
    pub fn get_user(&self) -> &ActorId {
        match self {
            WordleEvent::GameStarted { user } => user,
            WordleEvent::WordChecked { user, .. } => user,
        }
    }

    pub fn has_guessed(&self) -> bool {
        match self {
            WordleEvent::WordChecked {
                correct_positions, ..
            } => correct_positions == &vec![0, 1, 2, 3, 4],
            _ => false,
        }
    }
}

impl From<&WordleEvent> for GameSessionEvent {
    fn from(wordle_event: &WordleEvent) -> Self {
        match wordle_event {
            WordleEvent::GameStarted { .. } => GameSessionEvent::StartSuccess,
            WordleEvent::WordChecked {
                correct_positions,
                contained_in_word,
                ..
            } => GameSessionEvent::CheckWordResult {
                correct_positions: correct_positions.clone(),
                contained_in_word: contained_in_word.clone(),
            },
        }
    }
}

#[derive(Default, Debug, Clone, Encode, Decode, TypeInfo)]
pub struct SessionInfo {
    pub session_id: MessageId,
    pub original_msg_id: MessageId,
    pub send_to_wordle_msg_id: MessageId,
    pub tries: u8,
    pub session_status: SessionStatus,
}

impl SessionInfo {
    pub fn is_wait_reply_status(&self) -> bool {
        matches!(
            self.session_status,
            SessionStatus::WaitWordleCheckWordReply | SessionStatus::WaitWordleStartReply
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct GameSession {
    pub wordle_program_id: ActorId,
    pub sessions: HashMap<ActorId, SessionInfo>,
}

impl GameSession {
    pub fn start_game(&mut self, user: ActorId) {
        let session_info = self.sessions.entry(user).or_default();
        session_info.session_status = SessionStatus::WaitWordleStartReply;
        let send_to_wordle_msg_id = msg::send(
            self.wordle_program_id,
            WordleAction::StartGame { user },
            0,
        ).expect("Failed to send StartGame action");
        session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
        session_info.original_msg_id = msg::id();
    }

    pub fn check_word(&mut self, user: ActorId, word: String) {
        let session_info = self.sessions.entry(user).or_default();
        session_info.tries += 1;

        if word.len() == 5 && word.chars().all(|c| c.is_lowercase()) {
            let send_to_wordle_msg_id = msg::send(
                self.wordle_program_id,
                WordleAction::CheckWord { user, word },
                0,
            ).expect("Failed to send CheckWord action");

            session_info.send_to_wordle_msg_id = send_to_wordle_msg_id;
            session_info.session_status = SessionStatus::WaitWordleCheckWordReply;
        } else {
            panic!("Invalid word");
        }
    }
}

impl From<&GameSession> for GameSessionState {
    fn from(game_session: &GameSession) -> Self {
        Self {
            wordle_program_id: game_session.wordle_program_id,
            game_sessions: game_session
                .sessions
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
        }
    }
}

pub struct GameSessionMetadata;

impl Metadata for GameSessionMetadata {
    type Init = In<GameSessionInit>;
    type Handle = InOut<GameSessionAction, GameSessionEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = Out<GameSessionState>;
}
