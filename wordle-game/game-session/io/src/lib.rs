#![no_std]

use gmeta::{InOut, Metadata,In,Out};
use gstd::{prelude::*, ActorId,collections::HashMap};

pub struct GameSessionMetadata;

impl Metadata for GameSessionMetadata {
    type Init = In<ActorId>;
    type Handle = InOut<Action, Event>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}

#[derive(Default)]
struct GameSession {
    wordle_program: ActorId,
    games: HashMap<ActorId, GameStatus>,
}

#[derive(TypeInfo, Encode, Decode, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
struct GameStatus {
    word: Option<String>,
    attempts: u8,
    status: GameState,
}

#[derive(TypeInfo, Encode, Decode, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
enum GameState {
    NotStarted,
    InProgress,
    GameOver(Outcome),
}

#[derive(TypeInfo, Encode, Decode, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
enum Outcome {
    Win,
    Lose,
}

#[derive(TypeInfo, Encode, Decode, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Action {
    StartGame {
        user: ActorId,
    },
    CheckWord {
        user: ActorId,
        word: String,
    },
    CheckGameStatus {
        user: ActorId,
    },
    EndGame {
        user: ActorId,
    },
}

#[derive(TypeInfo, Encode, Decode, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    GameStarted {
        user: ActorId,
    },
    WordChecked {
        user: ActorId,
        correct_positions: Vec<u8>,
        contained_in_word: Vec<u8>,
    },
}
