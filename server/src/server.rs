use std::collections::HashMap;

use itertools::Itertools;
use types::*;

use anyhow::anyhow;
use anyhow::Result;

#[derive(Default, Debug)]
pub struct ServerData {
    pub games: Vec<GameData>,
    pub players: Vec<PlayerData>,
    pub rooms: Vec<RoomData>,

    next_player_id: PlayerId,
    next_game_id: GameId,
    next_room_id: RoomId,
}

impl ServerData {
    fn create_player(&mut self) -> PlayerId {
        let next_id = self.next_player_id;
        self.next_player_id += 1;
        next_id
    }

    pub fn create_player_with_name(&mut self, player_name: String) -> Result<PlayerData> {
        if self.players.iter().any(|player| player.name == player_name) {
            return Err(anyhow!("This name is already taken"));
        }

        let player_data = PlayerData {
            id: self.create_player(),
            name: player_name,
        };

        self.players.push(player_data.clone());
        Ok(player_data)
    }

    pub fn create_game(&mut self, room_data: RoomData) -> GameData {
        let game_id = self.next_game_id;
        self.next_game_id += 1;

        GameData {
            settings: room_data.settings,
            players: room_data
                .players
                .into_iter()
                .map(|player| (player, 0))
                .collect_vec(),
            id: game_id,
            current_round: RoundData {
                inputs: HashMap::new(),
                result: None,
            },
            round_history: vec![],
            status: GameStatus::Running,
        }
    }

    pub fn create_room(
        &mut self,
        player_id: PlayerId,
        room_name: String,
        settings: Option<GameSettings>,
    ) -> anyhow::Result<RoomData> {
        let player_data = self
            .players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        let room_id = self.next_room_id;
        self.next_room_id += 1;

        let room_data = RoomData {
            id: room_id,
            settings: settings.unwrap_or(GameSettings {
                kind: GameKind::RockPaperScissors,
                player_count: 2,
                end_condition: EndCondition::FirstToScore(3),
            }),
            players: vec![player_data.clone()],
            name: room_name,
        };

        self.rooms.push(room_data.clone());
        Ok(room_data)
    }

    pub fn join_room(&mut self, player_id: PlayerId, room_id: RoomId) -> Result<RoomData> {
        //Player must exist in players list
        let player_data = self
            .players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        //Room must exist in rooms list
        let room_data = self
            .rooms
            .iter_mut()
            .find(|room| room.id == room_id)
            .ok_or_else(|| anyhow!("Unknown room id"))?;

        if room_data
            .players
            .iter()
            .any(|player| player.id == player_id)
        {
            return Err(anyhow!("Player already in the room"));
        }

        if room_data.settings.player_count as usize <= room_data.players.len() {
            return Err(anyhow!("Room full"));
        }

        room_data.players.push(player_data.clone());

        Ok(room_data.clone())
    }

    pub fn leave_room(&mut self, player_id: PlayerId, room_id: RoomId) -> Result<()> {
        //Player must exist in players list
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        //Room must exist in rooms list
        let (room_index, room_data) = self
            .rooms
            .iter_mut()
            .enumerate()
            .find(|(_, room)| room.id == room_id)
            .ok_or_else(|| anyhow!("Unknown room id"))?;

        if !room_data
            .players
            .iter()
            .any(|player| player.id == player_id)
        {
            return Err(anyhow!("Player already left the room"));
        }

        room_data
            .players
            .retain_mut(|player| player.id != player_id);

        if room_data.players.is_empty() {
            self.rooms.remove(room_index);
        }

        Ok(())
    }

    pub fn get_room_data(&self, player_id: PlayerId, room_id: RoomId) -> Result<RoomData> {
        //Player must exist in players list
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        //Room must exist in rooms list
        let room_data = self
            .rooms
            .iter()
            .find(|room| room.id == room_id)
            .ok_or_else(|| anyhow!("Unknown room id"))?;

        if !room_data
            .players
            .iter()
            .any(|player| player.id == player_id)
        {
            return Err(anyhow!("Player not in the room"));
        }

        Ok(room_data.clone())
    }

    pub fn launch_room(&mut self, player_id: PlayerId, room_id: RoomId) -> Result<GameData> {
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        let (room_index, room_data) = self
            .rooms
            .iter()
            .enumerate()
            .find(|(_, room)| room.id == room_id)
            .ok_or_else(|| anyhow!("Unknown room id"))?;

        let (player_index, _) = room_data
            .players
            .iter()
            .enumerate()
            .find(|(_, player)| player.id == player_id)
            .ok_or_else(|| anyhow!("Player not in the room"))?;

        if player_index != 0 {
            return Err(anyhow!("Player is not the host"));
        }

        if room_data.players.len() != room_data.settings.player_count {
            return Err(anyhow!("Room must be full to launch the game"));
        }

        let game_data = self.create_game(room_data.clone());
        self.games.push(game_data.clone());

        self.rooms.remove(room_index);

        Ok(game_data)
    }

    pub fn get_game_data(&self, player_id: PlayerId, game_id: GameId) -> Result<GameData> {
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        let game_data = self
            .games
            .iter()
            .find(|game| game.id == game_id)
            .ok_or_else(|| anyhow!("Unknown game id"))?;

        if !game_data
            .players
            .iter()
            .any(|(player, _)| player.id == player_id)
        {
            return Err(anyhow!("Player not in the game"));
        }

        Ok(game_data.clone())
    }

    pub fn play_round(
        &mut self,
        player_id: PlayerId,
        game_id: GameId,
        action: ActionKind,
    ) -> Result<GameData> {
        self.players
            .iter()
            .find(|player| player.id == player_id)
            .ok_or_else(|| anyhow!("Unknown player id"))?;

        let game_data = self
            .games
            .iter_mut()
            .find(|game| game.id == game_id)
            .ok_or_else(|| anyhow!("Unknown game id"))?;

        if !game_data
            .players
            .iter()
            .any(|(player, _)| player.id == player_id)
        {
            return Err(anyhow!("Player not in the game"));
        }

        if game_data.status != GameStatus::Running {
            return Err(anyhow!("Game is not running anymore"));
        }

        game_data
            .current_round
            .inputs
            .entry(player_id)
            .and_modify(|e| *e = action.clone())
            .or_insert(action);

        if !game_data
            .players
            .iter()
            .any(|(player_data, _)| !game_data.current_round.inputs.contains_key(&player_data.id))
        {
            let mut round_results = Vec::new();

            let mut keys = game_data.current_round.inputs.keys();
            while let Some(first_player_id) = keys.next() {
                let iter = keys.clone();

                let p1_tuple = (
                    *first_player_id,
                    game_data.current_round.inputs.get(first_player_id).unwrap(),
                );

                for second_player_id in iter {
                    let p2_tuple = (
                        *second_player_id,
                        game_data
                            .current_round
                            .inputs
                            .get(second_player_id)
                            .unwrap(),
                    );

                    let round_result = match (p1_tuple.1, p2_tuple.1) {
                        (ActionKind::Rock, ActionKind::Rock)
                        | (ActionKind::Paper, ActionKind::Paper)
                        | (ActionKind::Scissors, ActionKind::Scissors) => RoundResult::Draw,
                        (ActionKind::Rock, ActionKind::Paper)
                        | (ActionKind::Paper, ActionKind::Scissors)
                        | (ActionKind::Scissors, ActionKind::Rock) => {
                            game_data
                                .players
                                .iter_mut()
                                .for_each(|(player_data, score)| {
                                    if player_data.id == p2_tuple.0 {
                                        *score += 1
                                    }
                                });
                            RoundResult::Winner(p2_tuple.0)
                        }
                        (ActionKind::Rock, ActionKind::Scissors)
                        | (ActionKind::Paper, ActionKind::Rock)
                        | (ActionKind::Scissors, ActionKind::Paper) => {
                            game_data
                                .players
                                .iter_mut()
                                .for_each(|(player_data, score)| {
                                    if player_data.id == p1_tuple.0 {
                                        *score += 1
                                    }
                                });
                            RoundResult::Winner(p1_tuple.0)
                        }
                    };

                    round_results.push(round_result.clone());
                }
            }
            game_data.current_round.result = Some(round_results.to_vec());

            game_data
                .round_history
                .push(game_data.current_round.clone());
            game_data.current_round = RoundData::default();

            match game_data.settings.end_condition {
                EndCondition::TotalRounds(x) => {
                    if game_data.round_history.len() == x {
                        game_data.status = GameStatus::Ended;
                    }
                }
                EndCondition::FirstToScore(x) => {
                    if let Some((_, max)) = game_data
                        .players
                        .iter()
                        .max_by(|(_, a_score), (_, b_score)| a_score.cmp(b_score))
                    {
                        if *max == x {
                            game_data.status = GameStatus::Ended;
                        }
                    }
                }
            }
        }

        Ok(game_data.clone())
    }

    pub fn get_rooms_list(&self) -> Vec<RoomData> {
        self.rooms.to_vec()
    }
}

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_create_player() {
        let mut server_data = ServerData::default();

        assert_eq!(
            server_data
                .create_player_with_name("Alice".to_string())
                .unwrap(),
            PlayerData {
                id: 0,
                name: "Alice".to_string()
            }
        );

        assert_eq!(
            server_data
                .create_player_with_name("Bob".to_string())
                .unwrap(),
            PlayerData {
                id: 1,
                name: "Bob".to_string()
            }
        );

        assert!(
            server_data
                .create_player_with_name("Bob".to_string())
                .is_err(),
            "Bob already exists"
        );
    }

    #[test]

    fn test_main_loop() {
        let mut server_data = ServerData::default();

        let alice = server_data
            .create_player_with_name("Alice".to_string())
            .unwrap();
        let bob = server_data
            .create_player_with_name("Bob".to_string())
            .unwrap();
        let charlie = server_data
            .create_player_with_name("Charlie".to_string())
            .unwrap();

        let room_data = server_data
            .create_room(
                alice.id,
                "test room".to_string(),
                Some(GameSettings {
                    kind: GameKind::RockPaperScissors,
                    player_count: 2,
                    end_condition: EndCondition::FirstToScore(2),
                }),
            )
            .unwrap();

        assert_eq!(server_data.get_rooms_list().len(), 1);

        //bob joins the room, which becomes full
        server_data.join_room(bob.id, room_data.id).unwrap();

        //charlie can't the room,as it is full
        assert!(server_data.join_room(charlie.id, room_data.id).is_err());

        //alice leaves the room, the host is now the second one who joined, which is bob
        server_data.leave_room(alice.id, room_data.id).unwrap();

        //alice can't leave the room twice
        assert!(server_data.leave_room(alice.id, room_data.id).is_err());

        //can't launch a game if the room is not full
        assert!(server_data.launch_room(bob.id, room_data.id).is_err());

        //charlie joins the room, which becomes full again
        server_data.join_room(charlie.id, room_data.id).unwrap();

        //charlie can't join the room twice, as he is already inside
        assert!(server_data.join_room(charlie.id, room_data.id).is_err());

        //bob can now launch the game, as the room is full
        let game_data = server_data.launch_room(bob.id, room_data.id).unwrap();

        //There are no more rooms available, as the game got launched
        assert_eq!(server_data.get_rooms_list().len(), 0);

        //Alice can't play as she is not part of the game
        assert!(server_data
            .play_round(alice.id, game_data.id, ActionKind::Paper)
            .is_err());

        //The game should be running

        assert_eq!(game_data.status, GameStatus::Running);

        //bob plays paper
        let game_data = server_data
            .play_round(bob.id, game_data.id, ActionKind::Paper)
            .unwrap();

        assert!(game_data.current_round.inputs.contains_key(&bob.id));
        assert!(!game_data.current_round.inputs.contains_key(&charlie.id));

        //bob changes its mind and plays Rock
        let game_data = server_data
            .play_round(bob.id, game_data.id, ActionKind::Rock)
            .unwrap();

        assert!(game_data.current_round.inputs.contains_key(&bob.id));
        assert!(!game_data.current_round.inputs.contains_key(&charlie.id));

        //charlie plays Scissors
        let game_data = server_data
            .play_round(charlie.id, game_data.id, ActionKind::Scissors)
            .unwrap();

        //The round is over, bob has won (rock beats scissors)
        assert_eq!(game_data.round_history.len(), 1);
        assert_eq!(
            *(game_data
                .round_history
                .last()
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .get(0)
                .unwrap()),
            RoundResult::Winner(bob.id)
        );

        //bob plays Scissors
        let game_data = server_data
            .play_round(bob.id, game_data.id, ActionKind::Scissors)
            .unwrap();

        //charlie plays scissors too
        let game_data = server_data
            .play_round(charlie.id, game_data.id, ActionKind::Scissors)
            .unwrap();

        //The round is over, it's a draw
        assert_eq!(game_data.round_history.len(), 2);
        assert_eq!(
            *(game_data
                .round_history
                .last()
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .get(0)
                .unwrap()),
            RoundResult::Draw
        );

        //bob plays Scissors
        let game_data = server_data
            .play_round(bob.id, game_data.id, ActionKind::Scissors)
            .unwrap();

        //charlie plays Paper
        let game_data = server_data
            .play_round(charlie.id, game_data.id, ActionKind::Paper)
            .unwrap();

        //Bob wins
        assert_eq!(game_data.round_history.len(), 3);
        assert_eq!(
            *(game_data
                .round_history
                .last()
                .unwrap()
                .result
                .as_ref()
                .unwrap()
                .get(0)
                .unwrap()),
            RoundResult::Winner(bob.id)
        );

        assert_eq!(game_data.status, GameStatus::Ended);

        //charlie can't play anymore, as the game has ended
        assert!(server_data
            .play_round(charlie.id, game_data.id, ActionKind::Paper)
            .is_err());
    }
}
