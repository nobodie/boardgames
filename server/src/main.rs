mod server;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use server::ServerData;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use types::{
    net::{
        GetGameQuery, JoinGetLeaveRoomQuery, JoinGetRoomResponse, LaunchGameQuery,
        LaunchGetGameResponse, NewPlayerQuery, NewPlayerResponse, NewRoomQuery, NewRoomResponse,
        PlayRoundQuery, RoomsListResponse,
    },
    ActionKind, GameData, GameId, GameSettings, PlayerData, PlayerId, RoomData, RoomId,
};

use anyhow::Result;

trait OptionResponse {
    fn or_not_found(self, type_name: &str) -> Response;
}

impl<T> OptionResponse for Option<T>
where
    Json<T>: IntoResponse,
{
    fn or_not_found(self, type_name: &str) -> Response {
        match self {
            Some(x) => Json(x).into_response(),
            None => (StatusCode::NOT_FOUND, format!("{type_name} not found")).into_response(),
        }
    }
}

struct ServerContext {
    server_data: Mutex<ServerData>,
}

impl ServerContext {
    pub fn new() -> Self {
        Self {
            server_data: Mutex::new(ServerData::default()),
        }
    }

    async fn with_data<T>(&self, func: impl FnOnce(&ServerData) -> T) -> T {
        let server_data = self.server_data.lock().await;
        func(&server_data)
    }

    async fn with_data_mut<T>(&self, func: impl FnOnce(&mut ServerData) -> T) -> T {
        let mut server_data = self.server_data.lock().await;
        func(&mut server_data)
    }

    pub async fn create_player_with_name(&self, player_name: String) -> Result<PlayerData> {
        self.with_data_mut(|server_data| server_data.create_player_with_name(player_name))
            .await
    }

    pub async fn get_rooms_list(&self) -> Vec<RoomData> {
        self.with_data(ServerData::get_rooms_list).await
    }

    pub async fn create_room(
        &self,
        player_id: PlayerId,
        room_name: String,
        settings: Option<GameSettings>,
    ) -> Result<RoomData> {
        self.with_data_mut(|server_data| server_data.create_room(player_id, room_name, settings))
            .await
    }

    pub async fn join_room(&self, player_id: PlayerId, room_id: RoomId) -> Result<RoomData> {
        self.with_data_mut(|server_data| server_data.join_room(player_id, room_id))
            .await
    }

    pub async fn leave_room(&self, player_id: PlayerId, room_id: RoomId) -> Result<()> {
        self.with_data_mut(|server_data| server_data.leave_room(player_id, room_id))
            .await
    }

    pub async fn get_room_data(&self, player_id: PlayerId, room_id: RoomId) -> Result<RoomData> {
        self.with_data(|server_data| server_data.get_room_data(player_id, room_id))
            .await
    }

    pub async fn launch_room(&self, player_id: PlayerId, room_id: RoomId) -> Result<GameData> {
        self.with_data_mut(|server_data| server_data.launch_room(player_id, room_id))
            .await
    }

    pub async fn get_game_data(&self, player_id: PlayerId, game_id: GameId) -> Result<GameData> {
        self.with_data(|server_data| server_data.get_game_data(player_id, game_id))
            .await
    }

    pub async fn play_round(
        &self,
        player_id: PlayerId,
        game_id: GameId,
        action: ActionKind,
    ) -> Result<GameData> {
        self.with_data_mut(|server_data| server_data.play_round(player_id, game_id, action))
            .await
    }
}

#[tokio::main]
async fn main() {
    let shared_context = Arc::new(ServerContext::new());
    let thread_server_context = shared_context.clone();

    let app = Router::new()
        .route("/player/new", get(new_player))
        .route("/rooms/list", get(rooms_list))
        .route("/room/new", get(new_room))
        .route("/room/join", get(join_room))
        .route("/room/leave", get(leave_room))
        .route("/room/data", get(get_room_data))
        .route("/room/launch", get(launch_room))
        .route("/game/data", get(get_game_data))
        .route("/game/play", get(play_round))
        .layer(CorsLayer::permissive())
        .with_state(shared_context.clone());

    let axum_lobby_handle = tokio::spawn(
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service()),
    );

    /*let app = Router::new()
        .route("/game/data", get(get_game_data))
        .layer(CorsLayer::permissive())
        .with_state(shared_context.clone());

    let axum_rps_handle = tokio::spawn(
        axum::Server::bind(&"0.0.0.0:3001".parse().unwrap()).serve(app.into_make_service()),
    );*/

    let log_feed_handle = tokio::spawn(async move {
        loop {
            {
                let server_data = thread_server_context.server_data.lock().await;

                println!("Total players count : {}", server_data.players.len());
                println!("Total rooms count : {}", server_data.rooms.len());
                println!("Total games count : {}", server_data.games.len());
            }
            std::thread::sleep(std::time::Duration::from_millis(5000));
        }
    });

    let _ = tokio::join!(axum_lobby_handle);
    //tokio::join!(axum_rps_handle);
    let _ = tokio::join!(log_feed_handle);
}

async fn new_player(
    new_player_query: Option<Query<NewPlayerQuery>>,
    State(ctx): State<Arc<ServerContext>>,
) -> Response {
    let player_name = if let Some(new_player_query) = new_player_query {
        new_player_query.name.clone()
    } else {
        "toto".to_string()
    };

    match ctx.create_player_with_name(player_name).await {
        Ok(player_data) => Json(NewPlayerResponse::from(player_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn rooms_list(State(ctx): State<Arc<ServerContext>>) -> Response {
    let rooms_list = ctx.get_rooms_list().await;
    Json(RoomsListResponse::from(rooms_list)).into_response()
}

async fn new_room(
    State(ctx): State<Arc<ServerContext>>,
    Query(new_room_query): Query<NewRoomQuery>,
) -> Response {
    match ctx
        .create_room(
            new_room_query.player_id,
            new_room_query.room_name,
            new_room_query.settings,
        )
        .await
    {
        Ok(room_data) => Json(NewRoomResponse::from(room_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn join_room(
    State(ctx): State<Arc<ServerContext>>,
    Query(join_room_query): Query<JoinGetLeaveRoomQuery>,
) -> Response {
    match ctx
        .join_room(join_room_query.player_id, join_room_query.room_id)
        .await
    {
        Ok(room_data) => Json(JoinGetRoomResponse::from(room_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn leave_room(
    State(ctx): State<Arc<ServerContext>>,
    Query(leave_room_query): Query<JoinGetLeaveRoomQuery>,
) -> Response {
    match ctx
        .leave_room(leave_room_query.player_id, leave_room_query.room_id)
        .await
    {
        Ok(_) => (StatusCode::OK, "Ok").into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn get_room_data(
    State(ctx): State<Arc<ServerContext>>,
    Query(get_room_data_query): Query<JoinGetLeaveRoomQuery>,
) -> Response {
    match ctx
        .get_room_data(get_room_data_query.player_id, get_room_data_query.room_id)
        .await
    {
        Ok(room_data) => Json(JoinGetRoomResponse::from(room_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn launch_room(
    State(ctx): State<Arc<ServerContext>>,
    Query(launch_game_query): Query<LaunchGameQuery>,
) -> Response {
    match ctx
        .launch_room(launch_game_query.player_id, launch_game_query.room_id)
        .await
    {
        Ok(game_data) => Json(LaunchGetGameResponse::from(game_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn get_game_data(
    State(ctx): State<Arc<ServerContext>>,
    Query(get_game_query): Query<GetGameQuery>,
) -> Response {
    match ctx
        .get_game_data(get_game_query.player_id, get_game_query.game_id)
        .await
    {
        Ok(game_data) => Json(LaunchGetGameResponse::from(game_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}

async fn play_round(
    State(ctx): State<Arc<ServerContext>>,
    Query(play_round_query): Query<PlayRoundQuery>,
) -> Response {
    match ctx
        .play_round(
            play_round_query.player_id,
            play_round_query.game_id,
            play_round_query.action,
        )
        .await
    {
        Ok(game_data) => Json(LaunchGetGameResponse::from(game_data)).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, e.to_string()).into_response(),
    }
}
