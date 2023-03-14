use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

pub type RoomId = i32;
pub type GameId = i32;
pub type PlayerId = i32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameKind {
    RockPaperScissors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EndCondition {
    TotalRounds(usize),
    FirstToScore(usize),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum GameStatus {
    Running,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ActionKind {
    Rock,
    Paper,
    Scissors,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    pub kind: GameKind,
    #[serde_as(as = "DisplayFromStr")]
    pub player_count: usize,
    pub end_condition: EndCondition,
}

#[derive(Debug, Clone)]
pub struct RoomData {
    pub id: RoomId,
    pub name: String,
    pub settings: GameSettings,
    pub players: Vec<PlayerData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerData {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum RoundResult {
    Draw,
    Winner(PlayerId),
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RoundData {
    pub inputs: HashMap<PlayerId, ActionKind>,
    pub result: Option<Vec<RoundResult>>,
}

#[derive(Debug, Clone)]
pub struct GameData {
    pub id: GameId,
    pub settings: GameSettings,
    pub players: Vec<(PlayerData, usize)>,
    pub current_round: RoundData,
    pub round_history: Vec<RoundData>,
    pub status: GameStatus,
}

pub mod net {

    use serde::{Deserialize, Serialize};

    use crate::{
        ActionKind, GameData, GameId, GameSettings, GameStatus, PlayerData, PlayerId, RoomData,
        RoomId, RoundData,
    };

    #[derive(Serialize, Debug, Clone)]
    pub struct PlayerFullData {
        id: PlayerId,
        name: String,
    }

    impl From<PlayerData> for PlayerFullData {
        fn from(value: PlayerData) -> Self {
            Self {
                id: value.id,
                name: value.name,
            }
        }
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct PlayerPublicData {
        name: String,
    }

    impl From<PlayerData> for PlayerPublicData {
        fn from(value: PlayerData) -> Self {
            Self { name: value.name }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct NewPlayerQuery {
        pub name: String,
    }

    #[derive(Serialize, Debug, Clone)]
    pub struct NewPlayerResponse {
        pub player: PlayerFullData,
    }

    impl From<PlayerData> for NewPlayerResponse {
        fn from(value: PlayerData) -> Self {
            Self {
                player: PlayerFullData::from(value),
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct RoomPublicData {
        id: RoomId,
        name: String,
        settings: GameSettings,
        players: Vec<PlayerPublicData>,
    }

    impl From<RoomData> for RoomPublicData {
        fn from(value: RoomData) -> Self {
            Self {
                id: value.id,
                settings: value.settings,
                players: value
                    .players
                    .into_iter()
                    .map(PlayerPublicData::from)
                    .collect(),
                name: value.name,
            }
        }
    }

    #[derive(Debug, Serialize)]
    pub struct RoomsListResponse {
        rooms: Vec<RoomPublicData>,
    }

    impl From<Vec<RoomData>> for RoomsListResponse {
        fn from(value: Vec<RoomData>) -> Self {
            Self {
                rooms: value.into_iter().map(RoomPublicData::from).collect(),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct NewRoomQuery {
        pub player_id: PlayerId,
        pub room_name: String,
        #[serde(flatten)]
        pub settings: Option<GameSettings>,
    }

    #[derive(Debug, Serialize)]
    pub struct NewRoomResponse {
        pub room: RoomPublicData,
    }

    impl From<RoomData> for NewRoomResponse {
        fn from(value: RoomData) -> Self {
            Self {
                room: RoomPublicData::from(value),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct JoinGetLeaveRoomQuery {
        pub player_id: PlayerId,
        pub room_id: RoomId,
    }

    #[derive(Debug, Serialize)]
    pub struct JoinGetRoomResponse {
        pub room: RoomPublicData,
    }

    impl From<RoomData> for JoinGetRoomResponse {
        fn from(value: RoomData) -> Self {
            Self {
                room: RoomPublicData::from(value),
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct LaunchGameQuery {
        pub player_id: PlayerId,
        pub room_id: RoomId,
    }
    #[derive(Debug, Serialize)]
    pub struct LaunchGetGameResponse {
        id: GameId,
        settings: GameSettings,
        status: GameStatus,
        players: Vec<(PlayerPublicData, usize)>,
        waiting_for_players: Vec<PlayerPublicData>,
        round_history: Vec<RoundData>,
    }

    impl From<GameData> for LaunchGetGameResponse {
        fn from(value: GameData) -> Self {
            let mut waiting_for_players: Vec<PlayerData> = value
                .players
                .iter()
                .map(|(player, _)| player.clone())
                .collect();

            waiting_for_players
                .retain(|player_data| !value.current_round.inputs.contains_key(&player_data.id));

            Self {
                id: value.id,
                players: value
                    .players
                    .into_iter()
                    .map(|(player, score)| (PlayerPublicData::from(player), score))
                    .collect(),
                settings: value.settings,
                round_history: value.round_history.to_vec(),
                waiting_for_players: waiting_for_players
                    .into_iter()
                    .map(PlayerPublicData::from)
                    .collect(),
                status: value.status,
            }
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct GetGameQuery {
        pub player_id: PlayerId,
        pub game_id: GameId,
    }

    #[derive(Debug, Deserialize)]
    pub struct PlayRoundQuery {
        pub player_id: PlayerId,
        pub game_id: GameId,
        pub action: ActionKind,
    }
}
