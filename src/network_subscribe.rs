use anyhow::Result;
use core_mechanics::{board::Board, original_game::get_original_game_board};
use futures_lite::StreamExt;
use iroh::PublicKey;
use iroh_gossip::net::{Event, GossipEvent, GossipReceiver, GossipSender};
use uuid::Uuid;

use crate::network_message::Message;

pub async fn subscribe_server_loop(
    mut receiver: GossipReceiver,
    sender: GossipSender,
    my_public_id: PublicKey,
) -> Result<()> {
    println!(">>> I WILL KEEP TRACK OF THE GAME (I'M SERVER)");

    let mut seats: Vec<PublicKey> = Vec::<PublicKey>::with_capacity(4);
    let mut is_game_running = false;

    let mut board = get_original_game_board(2);

    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            match Message::from_bytes(&msg.content)? {
                Message::Action {
                    from,
                    action,
                    message_id: _,
                } => {
                    println!("Processing a new player action: {:?}", action);
                    if !is_game_running {
                        println!("The game is not running");
                        let message = Message::Announcement {
                            from: my_public_id,
                            message: "The game hasn't started yet".into(),
                            message_id: Uuid::new_v4(),
                        };
                        sender.broadcast(message.to_vec().into()).await?;
                        continue;
                    }
                    let copy_board = board.clone();
                    let current_player = copy_board.get_who_is_playing_now();
                    let current_player_public_key: &PublicKey = seats
                        .get::<usize>((current_player.id.id() - 1).into())
                        .unwrap();

                    if current_player_public_key != &from {
                        println!("It is not the players turn yet");
                        let message = Message::Announcement {
                            from: my_public_id,
                            message: format!("It is {} turn now.", current_player_public_key),
                            message_id: Uuid::new_v4(),
                        };
                        sender.broadcast(message.to_vec().into()).await?;
                        continue;
                    }

                    match Board::do_action(board.clone(), &action) {
                        Ok(new_board_state) => {
                            board = new_board_state.clone();
                            println!(">>> Board state updated!");
                            let message = Message::BoardStateUpdated {
                                from: my_public_id,
                                board: new_board_state,
                                message_id: Uuid::new_v4(),
                            };
                            let message = serde_json::to_string(&message).unwrap();
                            let result = sender.broadcast(message.into()).await;

                            if let Err(e) = result {
                                println!("Error while sending board to players: {}", e);
                            } else {
                                println!("Seems like it worked to send as a serde json");
                            }
                        }
                        Err(action_fail) => {
                            println!("The player made an invalid action");
                            let msg = serde_json::to_string(&action_fail)?;
                            let message = Message::Announcement {
                                from: my_public_id,
                                message: msg,
                                message_id: Uuid::new_v4(),
                            };
                            sender.broadcast(message.to_vec().into()).await?;
                        }
                    }
                }
                Message::JoinTable {
                    from,
                    message_id: _,
                } => {
                    if seats.len() < 4 {
                        seats.push(from);
                    } else {
                        let message = Message::Announcement {
                            from: my_public_id,
                            message: "The table is full".into(),
                            message_id: Uuid::new_v4(),
                        };
                        sender.broadcast(message.to_vec().into()).await?;
                    }
                }
                Message::Announcement {
                    from: _,
                    message: _,
                    message_id: _,
                } => (),
                Message::StartGame {
                    from: _,
                    message_id: _,
                } => {
                    if seats.len() < 2 {
                        let message = Message::Announcement {
                            from: my_public_id,
                            message: format!("Not enough players (minimum 2, got {})", seats.len()),
                            message_id: Uuid::new_v4(),
                        };
                        sender.broadcast(message.to_vec().into()).await?;
                        continue;
                    }
                    if is_game_running {
                        let message = Message::Announcement {
                            from: my_public_id,
                            message: "Game is already running".into(),
                            message_id: Uuid::new_v4(),
                        };
                        sender.broadcast(message.to_vec().into()).await?;
                        continue;
                    }
                    is_game_running = true;
                    let number_of_players = seats.iter().fold(0, |acc, _| acc + 1);
                    board = get_original_game_board(number_of_players);
                    let message = Message::Announcement {
                        from: my_public_id,
                        message: format!("Starting a new game with {} players", number_of_players),
                        message_id: Uuid::new_v4(),
                    };
                    sender.broadcast(message.to_vec().into()).await?;

                    // Notify first player
                    let message = Message::Announcement {
                        from: my_public_id,
                        message: format!("Is your {} turn now", seats.first().unwrap()),
                        message_id: Uuid::new_v4(),
                    };
                    sender.broadcast(message.to_vec().into()).await?;
                }
                Message::BoardStateUpdated {
                    from: _,
                    board: _,
                    message_id: _,
                } => (),
            }
        }
    }
    Ok(())
}

pub async fn subscribe_client_loop(mut receiver: GossipReceiver) -> Result<()> {
    println!(">>> I WILL JUST PLAY THE GAME (I'M CLIENT)");

    while let Some(event) = receiver.try_next().await? {
        if let Event::Gossip(GossipEvent::Received(msg)) = event {
            match Message::from_bytes(&msg.content)? {
                Message::Action {
                    from,
                    action,
                    message_id: _,
                } => {
                    println!("> Got, from {} action {:?}", from, action)
                }
                Message::JoinTable {
                    from,
                    message_id: _,
                } => {
                    println!("> {} joined the table", from.fmt_short());
                }
                Message::Announcement {
                    from: _,
                    message,
                    message_id: _,
                } => {
                    println!(">>> Server: {}", message);
                }
                Message::StartGame {
                    from: _,
                    message_id: _,
                } => {
                    println!("> Someone wants to start the game");
                }
                Message::BoardStateUpdated {
                    from: _,
                    board,
                    message_id: _,
                } => {
                    println!(">>> Board UPDATED (Would have the board here)");
                    println!("{}", serde_json::to_string(&board).unwrap());
                }
            }
        }
    }
    Ok(())
}
