use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use shakmaty::{uci::Uci, Position};
use tokio::sync::Mutex;

use crate::{
    server_types::{EngineInfo, EngineRequest, EngineRequestError, EngineResponse, EngineResult},
    Engine,
};

pub async fn serve_engine<E: Engine + 'static>(engine: E) -> Router {
    Router::new()
        .route("/", get(get_info).post(handle_move))
        .with_state(Arc::new(Mutex::new(engine)))
}

async fn get_info<E: Engine>(State(_): State<Arc<Mutex<E>>>) -> Json<EngineInfo<E>> {
    Json(E::get_info())
}

async fn handle_move<E: Engine>(
    State(e): State<Arc<Mutex<E>>>,
    Json(request): Json<EngineRequest<E>>,
) -> EngineResult<E> {
    let observe_other_rand_used;
    let produce_rand_used;
    let observe_mine_rand_used;

    let mut state = request.engine_state;

    // If the move is a null move, skip processing it
    let their_move = request.r#move;
    let game_after = if their_move != Uci::Null {
        // Try parsing the UCI into a move.
        let user_move = match their_move.to_move(&request.game_before) {
            Ok(user_move) => user_move,
            Err(_) => {
                return EngineResult::RequestError(EngineRequestError::PositionMoveMismatch);
            }
        };

        // Apply the move to the board.
        let mut game_after = request.game_before.clone();
        game_after.play_unchecked(&user_move);

        // The engine needs to observe this move.
        {
            let observe_rand = if let Some(v) = request.observe_mine_rand {
                v
            } else {
                rand::random()
            };
            observe_other_rand_used = Some(observe_rand);
            let mut engine = e.lock().await;
            if let Err(why) = engine
                .observe_move(observe_rand, &mut state, &user_move, &game_after)
                .await
            {
                return EngineResult::EngineError(why);
            }

            game_after
        }
    } else {
        // If the move is a null move, there is nothing to observe.
        observe_other_rand_used = None;
        request.game_before.clone()
    };

    // Now that the other move has been observed, we need to produce a new move.

    produce_rand_used = request.produce_rand.unwrap_or_else(rand::random);
    let (proposed_move, info) = {
        let mut engine = e.lock().await;
        if request.with_status_info {
            match engine
                .propose_move(produce_rand_used, &state, &game_after)
                .await
            {
                Ok((a, b)) => (a, Some(b)),
                Err(why) => return EngineResult::EngineError(why),
            }
        } else {
            match engine
                .propose_move_without_info(produce_rand_used, &state, &game_after)
                .await
            {
                Ok(a) => (a, None),
                Err(why) => return EngineResult::EngineError(why),
            }
        }
    };

    // Finally, observe our own move.

    observe_mine_rand_used = request.observe_your_rand.unwrap_or_else(rand::random);
    let game_after_mine = match game_after.clone().play(&proposed_move) {
        Ok(v) => v,
        Err(_) => {
            return EngineResult::RequestError(EngineRequestError::EngineSentIllegalMove {
                r#move: proposed_move.to_uci(shakmaty::CastlingMode::Standard),
            });
        }
    };
    {
        let mut engine = e.lock().await;
        if let Err(why) = engine
            .observe_move(
                observe_mine_rand_used,
                &mut state,
                &proposed_move,
                &game_after_mine,
            )
            .await
        {
            return EngineResult::EngineError(why);
        }
    }

    // Now that the move was produced and observed, construct a response.
    EngineResult::Ok(EngineResponse {
        r#move: proposed_move.to_uci(shakmaty::CastlingMode::Standard),
        game_after: game_after_mine,
        status_info: info,
        observe_other_rand_used,
        produce_rand_used,
        observe_mine_rand_used,
        engine_state: state,
    })
}
