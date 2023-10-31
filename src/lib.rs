pub mod chess_serde;
#[cfg(feature = "server")]
pub mod server;
pub mod server_types;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use server_types::EngineInfo;
use shakmaty::{Chess, Move};

pub use shakmaty;

/// The trait that defines a chess engine.
///
/// A chess engine is a program that takes board positions and produces moves.
///
/// An engine is provided a random number for every call to [`Engine::propose_move`] and [`Engine::observe_move`].
/// It should use that for any randomness needed in its calculation, for reproducibility.
/// If there is not enough random bits inside the number, an RNG should be seeded from it.
///
/// When the engine is playing as White, the game loop is as follows:
///
/// 1. An `Engine` is initialized.
/// 2. [`Engine::propose_move`] is called
/// 3. The move that White ends up playing is passed to [`Engine::observe_move`]
/// 4. The move that Black plays is passed to `observe_move`
/// 5. Repeat 2-4.
///
/// If the engine is playing as Black, before step 2, `observe_move` is called on White's move.
///
///
/// ## Statefulness
/// The engine must not store any important game info inside its own struct;
/// any state it needs for move correlation must be in the `State`, which can be round-tripped to the user.
/// In particular, the engine is supposed to make the same moves whether used multiple times or re-created,
/// as long as the `State`` is the same.
#[async_trait]
pub trait Engine: Send + Sync + Sized {
    /// An engine's state is the information it needs in order to produce moves.
    /// It is provided to the engine every time it is asked to make a move,
    /// but it will be stored externally.
    ///
    /// The Default implementation should correspond to a game state of the initial position, with white to move.
    type State: Serialize + DeserializeOwned + Default + Clone + Send + Sync + std::fmt::Debug;

    /// An engine may produce some kind of status information that explains its thinking process.
    type StatusInfo: std::fmt::Debug + Serialize + DeserializeOwned + Clone + Send + Sync;

    /// If an engine's thinking can fail, this type should explain how.
    ///
    /// When the engine returns this, the relevant operation is retried a few times.
    /// If it fails then, the game is considered forfeit by the engine.
    type Error: std::error::Error + Clone + Send + Sync;

    fn get_info() -> EngineInfo<Self>;

    /// Calculate a move for the current state.
    ///  
    /// In order to support stateless engines, the current [`Position`] is also provided.
    /// If the `State` disagrees with the `Position`, then this needs to return an error.
    ///
    /// Note that this is not necessarily the move that will be played.
    /// The engine will be told what move was actually played with [`Engine::observe_move`].
    async fn propose_move(
        &mut self,
        rand: u64,
        current_state: &Self::State,
        current_position: &Chess,
    ) -> Result<(Move, Self::StatusInfo), Self::Error>;

    /// Calculate a move without status info.
    ///
    /// The default implementation forwards to [`Self::propose_move`], but it can be overridden if there is efficiency gains to be had from omitting it.
    async fn propose_move_without_info(
        &mut self,
        rand: u64,
        current_state: &Self::State,
        current_position: &Chess,
    ) -> Result<Move, Self::Error> {
        self.propose_move(rand, current_state, current_position)
            .await
            .and_then(|v| Ok(v.0))
    }

    /// Observe that a move has occurred.
    /// This is called both for my own moves and for the opponent's moves.
    ///
    /// The provided [`Position`] already has the move applied to it.
    async fn observe_move(
        &mut self,
        rand: u64,
        state: &mut Self::State,
        move_taken: &Move,
        position_after: &Chess,
    ) -> Result<(), Self::Error>;
}
