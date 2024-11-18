pub struct Game {
    stream1: crate::Stream,
    stream2: crate::Stream,

    board1: logic::Board,
    board2: logic::Board,
    turn: u8,
}

impl Game {
    pub async fn new(
        mut stream1: crate::Stream,
        mut stream2: crate::Stream,
    ) -> Result<Game, crate::stream::Error> {
        Ok(Game {
            board1: logic::Board::from_ships(stream1.request_board().await?),
            board2: logic::Board::from_ships(stream2.request_board().await?),
            stream1,
            stream2,
            turn: 0,
        })
    }

    pub fn split_player_streams(&mut self) -> (&mut crate::Stream, &mut crate::Stream) {
        if self.turn % 2 == 0 {
            (&mut self.stream1, &mut self.stream2)
        } else {
            (&mut self.stream2, &mut self.stream1)
        }
    }

    pub fn split_player_boards(&mut self) -> (&mut logic::Board, &mut logic::Board) {
        if self.turn % 2 == 0 {
            (&mut self.board1, &mut self.board2)
        } else {
            (&mut self.board2, &mut self.board1)
        }
    }

    async fn play_turn(&mut self) -> Result<bool, crate::stream::Error> {
        let (player, opponent) = self.split_player_streams();
        let (target, success) = tokio::join!(
            player.request_target(),
            opponent.request_inform_target_selection(),
        );
        let target = target?;
        success?;

        let (_, opponent_board) = self.split_player_boards();
        let attack_info = opponent_board.target(target)?;

        let (player, opponent) = self.split_player_streams();
        let (success1, success2) = tokio::join!(
            player.request_inform_attack_info_opponent(attack_info, target),
            opponent.request_inform_attack_info_client(attack_info, target),
        );
        success1?;
        success2?;

        match attack_info {
            logic::board::AttackInfo::Hit(None) => Ok(true),
            logic::board::AttackInfo::Miss => {
                self.turn += 1;
                Ok(true)
            }

            logic::board::AttackInfo::Hit(Some(_)) => {
                let (_, opponent_board) = self.split_player_boards();
                if opponent_board.is_all_sunken() {
                    let (player, opponent) = self.split_player_streams();
                    let (success1, success2) = tokio::join!(
                        player.request_inform_victory(),
                        opponent.request_inform_loss(),
                    );
                    success1?;
                    success2?;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
        }
    }

    pub async fn play(mut self) -> Result<(), crate::stream::Error> {
        loop {
            match self.play_turn().await {
                Ok(true) => continue,
                Ok(false) => break Ok(()),
                Err(err) => break Err(err),
            }
        }
    }
}
