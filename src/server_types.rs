#[cfg(feature = "server")]
use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shakmaty::{uci::Uci, Chess};

use crate::Engine;

/// Request the engine to take a move.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineRequest<E: Engine> {
    /// The move that the user took. Put a null move here if the engine is making the first move.
    #[serde(with = "crate::chess_serde::uci_serde")]
    pub r#move: Uci,

    /// The game state before the move was played
    #[serde(with = "crate::chess_serde::position_serde")]
    pub game_before: Chess,

    /// The engine's internal state after its last move.
    pub engine_state: E::State,

    /// What random number to give to the engine when observing this move?
    /// If None, it will be generated.
    pub observe_mine_rand: Option<u64>,

    /// What random number to give to the engine when producing a new move?
    /// If None, it will be generated.
    pub produce_rand: Option<u64>,

    /// What random number to give to the engine when observing the engine's own move?
    /// If None, it will be generated.
    pub observe_your_rand: Option<u64>,

    /// Should status info be returned?
    pub with_status_info: bool,
}

/// General engine info, including initial state.
#[derive(Serialize, Deserialize)]
pub struct EngineInfo<E: Engine> {
    /// The engine's algorithm ID.
    pub id: String,

    /// A human-readable description of what the engine does.
    pub description: String,

    /// Initial state value. Pass this when making a move.
    pub initial_state: E::State,
}

/// Type-erased [`EngineInfo`], where the engine-specific fields have been replaced with [`serde_json::Value`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnyEngineInfo {
    /// The engine's algorithm ID.
    pub id: String,

    /// A human-readable description of what the engine does.
    pub description: String,

    /// Initial state value. Pass this when making a move.
    pub initial_state: Value,
}

/// Errors relating to a submitted request, independent of the engine.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[non_exhaustive]
pub enum EngineRequestError {
    /// The provided move is not legal in the provided position, or not at all.
    PositionMoveMismatch,

    /// The engine has generated a move that is not legal in the corresponding position.
    /// This is a bug in the engine.
    /// The suggested move is included.
    EngineSentIllegalMove {
        #[serde(with = "crate::chess_serde::uci_serde")]
        r#move: Uci,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngineResponse<E: Engine> {
    /// The move that the engine chose.
    #[serde(with = "crate::chess_serde::uci_serde")]
    pub r#move: Uci,

    /// The game state after this move was played.
    #[serde(with = "crate::chess_serde::position_serde")]
    pub game_after: Chess,

    /// The engine's status info about this move.
    /// It is None if the request asked for no status info.
    pub status_info: Option<E::StatusInfo>,

    /// The random number we gave to the engine when it was observing the previous move.
    /// None if it did not observe the previous move.
    pub observe_other_rand_used: Option<u64>,

    /// The random number we gave to the engine when it was producing this move.
    pub produce_rand_used: u64,

    /// The random number used to observe the move the engine had made.
    pub observe_mine_rand_used: u64,

    /// The engine's state. You need to pass this again if you want to continue this game.
    pub engine_state: E::State,
}

/// Type-erased [`EngineResponse`], where the engine-specific fields have been replaced with [`serde_json::Value`].
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnyEngineResponse {
    /// The move that the engine chose.
    #[serde(with = "crate::chess_serde::uci_serde")]
    pub r#move: Uci,

    /// The game state after this move was played.
    #[serde(with = "crate::chess_serde::position_serde")]
    pub game_after: Chess,

    /// The engine's status info about this move.
    /// It is None if the request asked for no status info.
    pub status_info: Option<Value>,

    /// The random number we gave to the engine when it was observing the previous move.
    /// None if it did not observe the previous move.
    pub observe_other_rand_used: Option<u64>,

    /// The random number we gave to the engine when it was producing this move.
    pub produce_rand_used: u64,

    /// The random number used to observe the move the engine had made.
    pub observe_mine_rand_used: u64,

    /// The engine's state. You need to pass this again if you want to continue this game.
    pub engine_state: Value,
}

#[derive(Clone, Debug)]
pub enum EngineResult<E: Engine> {
    RequestError(EngineRequestError),
    EngineError(E::Error),
    Ok(EngineResponse<E>),
}

/// Type-erased [`EngineResult`], where the engine-specific fields have been replaced with [`serde_json::Value`].
#[derive(Clone, Debug)]
pub enum AnyEngineResult {
    RequestError(EngineRequestError),
    EngineError(Value),
    Ok(AnyEngineResponse),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineInternalError {
    pub error_text: String,
}

#[cfg(feature = "server")]
impl<E> IntoResponse for EngineResult<E>
where
    E: Engine,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            EngineResult::RequestError(what) => {
                (StatusCode::BAD_REQUEST, Json(what)).into_response()
            }
            EngineResult::EngineError(what) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(EngineInternalError {
                    error_text: what.to_string(),
                }),
            )
                .into_response(),
            EngineResult::Ok(what) => (StatusCode::OK, Json(what)).into_response(),
        }
    }
}
