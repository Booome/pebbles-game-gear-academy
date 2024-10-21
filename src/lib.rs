#![no_std]

use gstd::{msg, Box};
use pebbles_game_io::*;
use rng::{MockRng, Rng};

static mut PEBBLES_GAME: Option<PebblesGame> = None;

#[no_mangle]
extern "C" fn init() {
    let pebbles_init: PebblesInit = msg::load().expect("Cannot load PebblesInit");
    let rng: Option<Box<dyn Rng>> = if pebbles_init.mock_rng.is_some() {
        Some(Box::new(MockRng::new(pebbles_init.mock_rng.unwrap())))
    } else {
        None
    };

    unsafe {
        PEBBLES_GAME = Some(PebblesGame::new(
            pebbles_init.difficulty,
            pebbles_init.pebbles_count,
            pebbles_init.max_pebbles_per_turn,
            rng,
        ));
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
extern "C" fn handle() {
    let pebbles_action: PebblesAction = msg::load().expect("Cannot load PeblesAction");

    let game_state = unsafe { PEBBLES_GAME.as_mut().unwrap() };

    match pebbles_action {
        PebblesAction::Turn(count) => game_state.user_turn(&count),

        PebblesAction::GiveUp => game_state.giveup(),

        PebblesAction::Restart {
            difficulty,
            pebbles_count,
            max_pebbles_per_turn,
            mock_rng,
        } => unsafe {
            let rng: Option<Box<dyn Rng>> = if mock_rng.is_some() {
                Some(Box::new(MockRng::new(mock_rng.unwrap())))
            } else {
                None
            };

            PEBBLES_GAME = Some(PebblesGame::new(
                difficulty,
                pebbles_count,
                max_pebbles_per_turn,
                rng,
            ));
        },
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
extern "C" fn state() {
    let game_state = unsafe { &PEBBLES_GAME.as_ref().unwrap().game_state };

    msg::reply(game_state, 0).expect("Cannot reply");
}
