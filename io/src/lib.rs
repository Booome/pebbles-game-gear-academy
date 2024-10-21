#![no_std]

use gmeta::{In, InOut, Metadata, Out};
use gstd::{msg, prelude::*};
use rng::{RealRng, Rng};

pub struct PebblesMetadata;

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct PebblesInit {
    pub difficulty: DifficultyLevel,
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub mock_rng: Option<Vec<u32>>,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum DifficultyLevel {
    #[default]
    Easy,
    Hard,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesAction {
    Turn(u32),
    GiveUp,
    Restart {
        difficulty: DifficultyLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
        mock_rng: Option<Vec<u32>>,
    },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo)]
pub enum PebblesEvent {
    CounterTurn(u32),
    Won(Player),
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum Player {
    #[default]
    User,
    Program,
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
pub struct GameState {
    pub pebbles_count: u32,
    pub max_pebbles_per_turn: u32,
    pub pebbles_remaining: u32,
    pub difficulty: DifficultyLevel,
    pub first_player: Player,
    pub winner: Option<Player>,
}

pub struct PebblesGame {
    pub game_state: GameState,
    pub rng: Box<dyn Rng>,
}

impl Metadata for PebblesMetadata {
    type Init = In<PebblesInit>;
    type Handle = InOut<PebblesAction, PebblesEvent>;
    type State = Out<GameState>;
    type Reply = ();
    type Others = ();
    type Signal = ();
}

impl PebblesGame {
    pub fn new(
        difficulty: DifficultyLevel,
        pebbles_count: u32,
        max_pebbles_per_turn: u32,
        rng: Option<Box<dyn Rng>>,
    ) -> Self {
        if (pebbles_count == 0) || (max_pebbles_per_turn == 0) {
            panic!("Invalid game initialization");
        }

        let mut rng = rng.unwrap_or_else(|| Box::new(RealRng));
        let first_player = match rng.gen() % 2 {
            0 => Player::User,
            1 => Player::Program,
            _ => unreachable!(),
        };
        let game_state = GameState {
            pebbles_count,
            max_pebbles_per_turn,
            pebbles_remaining: pebbles_count,
            difficulty,
            first_player,
            ..Default::default()
        };
        let mut pebbles_game = PebblesGame { game_state, rng };

        if pebbles_game.game_state.first_player == Player::Program {
            pebbles_game.program_turn();
        }

        pebbles_game
    }

    pub fn user_turn(&mut self, count: &u32) {
        if self.game_state.winner.is_some() {
            panic!("Game is over");
        }
        if *count > self.game_state.max_pebbles_per_turn
            || *count > self.game_state.pebbles_remaining
            || *count == 0
        {
            panic!("Invalid count");
        }

        self.process_turn(&Player::Program, count);

        if self.game_state.winner.is_none() {
            self.program_turn();
        }
    }

    pub fn giveup(&mut self) {
        if self.game_state.winner.is_some() {
            panic!("Game is over");
        }

        self.game_state.winner = Some(Player::Program);
        msg::reply(PebblesEvent::Won(Player::User), 0).expect("Failed to reply");
    }

    fn program_turn(&mut self) {
        let count = match self.game_state.difficulty {
            DifficultyLevel::Easy => {
                if self.game_state.pebbles_remaining < self.game_state.max_pebbles_per_turn {
                    self.game_state.pebbles_remaining.clone()
                } else {
                    self.rng.gen() % (self.game_state.max_pebbles_per_turn - 1) + 1
                }
            }
            DifficultyLevel::Hard => {
                let grundy =
                    self.game_state.pebbles_remaining % self.game_state.max_pebbles_per_turn + 1;
                if grundy != 0 {
                    grundy
                } else {
                    self.rng.gen() % (self.game_state.max_pebbles_per_turn - 1) + 1
                }
            }
        };

        self.process_turn(&Player::Program, &count);
    }

    fn process_turn(&mut self, player: &Player, count: &u32) {
        self.game_state.pebbles_remaining -= *count;

        if self.game_state.pebbles_remaining == 0 {
            self.game_state.winner = Some(player.clone());
            msg::reply(PebblesEvent::Won(player.clone()), 0).expect("Cannot reply");
        } else {
            msg::reply(PebblesEvent::CounterTurn(*count), 0).expect("Cannot reply");
        }
    }
}
