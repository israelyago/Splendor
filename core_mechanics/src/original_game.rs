use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::vec;

use crate::bank::Funds;
use crate::board::Board;
use crate::board::ProductionTier;
use crate::noble::Noble;
use crate::noble::NobleId;
use crate::piece::Piece;
use crate::player::Player;
use crate::player::PlayerId;
use crate::production_card::CardId;
use crate::production_card::Identifiable;
use crate::production_card::ProductionCard;

pub fn get_original_game_board(n_of_players: u8) -> Board {
    let allowed_n_of_players = 2..=4;
    if !allowed_n_of_players.contains(&n_of_players) {
        panic!(
            "The original game is only defined for 2 to 4 players. '{:?}' given",
            n_of_players
        )
    }
    let mut players = vec![];
    let empty_funds = &Funds::new(0, 0, 0, 0, 0, 0);
    for n in 1..=n_of_players {
        players.push(Player::new(
            PlayerId::new(n),
            empty_funds.clone(),
            vec![],
            vec![],
        ));
    }
    let mut bank = Funds::new(7, 7, 7, 7, 7, 5);
    if n_of_players == 3 {
        bank = Funds::new(5, 5, 5, 5, 5, 5);
    }
    if n_of_players == 2 {
        bank = Funds::new(4, 4, 4, 4, 4, 5);
    }

    let decks = get_shuffled_decks();
    let nobles = get_random_nobles(n_of_players + 1);

    Board::new(players, bank, decks, nobles)
}

fn get_shuffled_decks() -> HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>> {
    let mut unique_id = 0;
    let tier_one: Vec<Identifiable<ProductionCard, CardId>> = get_tier_one_cards()
        .iter()
        .map(|c| {
            unique_id += 1;
            Identifiable::new(c.clone(), CardId::new(unique_id))
        })
        .collect();

    let tier_two: Vec<Identifiable<ProductionCard, CardId>> = get_tier_two_cards()
        .iter()
        .map(|c| {
            unique_id += 1;
            Identifiable::new(c.clone(), CardId::new(unique_id))
        })
        .collect();

    let tier_three: Vec<Identifiable<ProductionCard, CardId>> = get_tier_three_cards()
        .iter()
        .map(|c| {
            unique_id += 1;
            Identifiable::new(c.clone(), CardId::new(unique_id))
        })
        .collect();

    let mut decks: HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>> =
        HashMap::new();
    decks.insert(ProductionTier::One, shuffle_vec(tier_one));
    decks.insert(ProductionTier::Two, shuffle_vec(tier_two));
    decks.insert(ProductionTier::Three, shuffle_vec(tier_three));

    decks
}

fn shuffle_vec<T: Clone>(v: Vec<T>) -> Vec<T> {
    let rng = &mut rand::thread_rng();
    v.choose_multiple(rng, v.len()).cloned().collect()
}

fn get_random_nobles(quantity: u8) -> Vec<Noble> {
    let nobles = get_nobles();
    let rng = &mut rand::thread_rng();
    nobles
        .choose_multiple(rng, quantity as usize)
        .cloned()
        .collect()
}

fn get_tier_one_cards() -> Vec<ProductionCard> {
    vec![
        ProductionCard::new(Funds::new(0, 0, 2, 0, 2, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(1, 0, 1, 2, 1, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(1, 0, 1, 1, 1, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(0, 3, 0, 0, 0, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(0, 0, 4, 0, 0, 0), Piece::Brown, Some(1)),
        ProductionCard::new(Funds::new(1, 1, 2, 0, 1, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(1, 3, 1, 0, 0, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(2, 1, 0, 1, 1, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(0, 2, 0, 2, 0, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(0, 0, 2, 2, 0, 0), Piece::White, None),
        ProductionCard::new(Funds::new(0, 0, 0, 0, 3, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(0, 0, 0, 4, 0, 0), Piece::Green, Some(1)),
        ProductionCard::new(Funds::new(0, 1, 3, 0, 1, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(2, 0, 1, 2, 0, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(1, 0, 0, 3, 1, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(0, 0, 0, 0, 4, 0), Piece::Red, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 3, 0, 0, 0), Piece::White, None),
        ProductionCard::new(Funds::new(2, 2, 0, 0, 0, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(3, 1, 0, 1, 0, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(0, 2, 0, 0, 2, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(1, 1, 0, 1, 1, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(4, 0, 0, 0, 0, 0), Piece::Blue, Some(1)),
        ProductionCard::new(Funds::new(0, 1, 0, 2, 2, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(2, 0, 0, 0, 2, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(0, 1, 2, 0, 0, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(1, 0, 2, 0, 2, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(2, 2, 0, 0, 1, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(0, 0, 0, 3, 0, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(0, 0, 2, 1, 2, 0), Piece::White, None),
        ProductionCard::new(Funds::new(1, 1, 1, 1, 0, 0), Piece::White, None),
        ProductionCard::new(Funds::new(0, 0, 0, 2, 1, 0), Piece::Blue, None),
        ProductionCard::new(Funds::new(1, 1, 1, 0, 1, 0), Piece::Brown, None),
        ProductionCard::new(Funds::new(2, 0, 2, 0, 0, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(3, 0, 0, 0, 0, 0), Piece::Green, None),
        ProductionCard::new(Funds::new(1, 2, 1, 1, 0, 0), Piece::White, None),
        ProductionCard::new(Funds::new(2, 0, 0, 1, 0, 0), Piece::White, None),
        ProductionCard::new(Funds::new(0, 0, 1, 1, 3, 0), Piece::White, None),
        ProductionCard::new(Funds::new(0, 4, 0, 0, 0, 0), Piece::White, Some(1)),
        ProductionCard::new(Funds::new(0, 1, 1, 1, 2, 0), Piece::Red, None),
        ProductionCard::new(Funds::new(0, 1, 1, 1, 1, 0), Piece::Red, None),
    ]
}
fn get_tier_two_cards() -> Vec<ProductionCard> {
    vec![
        ProductionCard::new(Funds::new(0, 3, 0, 2, 3, 0), Piece::Brown, Some(1)),
        ProductionCard::new(Funds::new(3, 2, 0, 0, 3, 0), Piece::Green, Some(1)),
        ProductionCard::new(Funds::new(2, 0, 3, 3, 0, 0), Piece::Red, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 6, 0, 0, 0), Piece::Blue, Some(3)),
        ProductionCard::new(Funds::new(1, 0, 0, 4, 2, 0), Piece::Blue, Some(2)),
        ProductionCard::new(Funds::new(3, 0, 3, 0, 2, 0), Piece::White, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 2, 1, 4, 0), Piece::Green, Some(2)),
        ProductionCard::new(Funds::new(0, 0, 5, 0, 0, 0), Piece::Blue, Some(2)),
        ProductionCard::new(Funds::new(0, 0, 0, 0, 5, 0), Piece::Brown, Some(2)),
        ProductionCard::new(Funds::new(2, 0, 0, 3, 2, 0), Piece::Red, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 0, 0, 6, 0), Piece::White, Some(3)),
        ProductionCard::new(Funds::new(0, 2, 4, 0, 1, 0), Piece::Red, Some(2)),
        ProductionCard::new(Funds::new(5, 0, 0, 0, 0, 0), Piece::White, Some(2)),
        ProductionCard::new(Funds::new(0, 6, 0, 0, 0, 0), Piece::Green, Some(3)),
        ProductionCard::new(Funds::new(0, 5, 0, 0, 0, 0), Piece::Green, Some(2)),
        ProductionCard::new(Funds::new(0, 0, 0, 5, 0, 0), Piece::Red, Some(2)),
        ProductionCard::new(Funds::new(0, 2, 2, 0, 3, 0), Piece::Brown, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 0, 6, 0, 0), Piece::Brown, Some(3)),
        ProductionCard::new(Funds::new(3, 5, 0, 0, 0, 0), Piece::Brown, Some(2)),
        ProductionCard::new(Funds::new(0, 3, 5, 0, 0, 0), Piece::Green, Some(2)),
        ProductionCard::new(Funds::new(0, 3, 2, 3, 0, 0), Piece::Blue, Some(1)),
        ProductionCard::new(Funds::new(2, 2, 2, 0, 0, 0), Piece::Blue, Some(1)),
        ProductionCard::new(Funds::new(0, 0, 3, 0, 5, 0), Piece::Blue, Some(2)),
        ProductionCard::new(Funds::new(0, 0, 3, 2, 2, 0), Piece::Green, Some(1)),
        ProductionCard::new(Funds::new(5, 0, 0, 3, 0, 0), Piece::White, Some(2)),
        ProductionCard::new(Funds::new(4, 1, 0, 2, 0, 0), Piece::White, Some(2)),
        ProductionCard::new(Funds::new(2, 4, 0, 1, 0, 0), Piece::Brown, Some(2)),
        ProductionCard::new(Funds::new(2, 3, 0, 2, 0, 0), Piece::White, Some(1)),
        ProductionCard::new(Funds::new(6, 0, 0, 0, 0, 0), Piece::Red, Some(3)),
        ProductionCard::new(Funds::new(0, 0, 0, 5, 3, 0), Piece::Red, Some(2)),
    ]
}
fn get_tier_three_cards() -> Vec<ProductionCard> {
    vec![
        ProductionCard::new(Funds::new(3, 0, 3, 3, 5, 0), Piece::Green, Some(3)),
        ProductionCard::new(Funds::new(3, 3, 0, 5, 3, 0), Piece::Blue, Some(3)),
        ProductionCard::new(Funds::new(0, 3, 6, 0, 3, 0), Piece::Green, Some(4)),
        ProductionCard::new(Funds::new(0, 0, 0, 7, 3, 0), Piece::White, Some(5)),
        ProductionCard::new(Funds::new(7, 0, 0, 0, 0, 0), Piece::Brown, Some(4)),
        ProductionCard::new(Funds::new(6, 3, 0, 3, 0, 0), Piece::Brown, Some(4)),
        ProductionCard::new(Funds::new(0, 0, 3, 3, 6, 0), Piece::Blue, Some(4)),
        ProductionCard::new(Funds::new(0, 7, 0, 0, 0, 0), Piece::Red, Some(4)),
        ProductionCard::new(Funds::new(0, 3, 5, 3, 3, 0), Piece::Red, Some(3)),
        ProductionCard::new(Funds::new(3, 6, 3, 0, 0, 0), Piece::Red, Some(4)),
        ProductionCard::new(Funds::new(3, 0, 0, 6, 3, 0), Piece::White, Some(4)),
        ProductionCard::new(Funds::new(3, 5, 3, 0, 3, 0), Piece::Brown, Some(3)),
        ProductionCard::new(Funds::new(0, 0, 3, 0, 7, 0), Piece::Blue, Some(5)),
        ProductionCard::new(Funds::new(3, 7, 0, 0, 0, 0), Piece::Red, Some(5)),
        ProductionCard::new(Funds::new(0, 3, 7, 0, 0, 0), Piece::Green, Some(5)),
        ProductionCard::new(Funds::new(0, 0, 0, 7, 0, 0), Piece::White, Some(4)),
        ProductionCard::new(Funds::new(0, 0, 7, 0, 0, 0), Piece::Green, Some(4)),
        ProductionCard::new(Funds::new(5, 3, 3, 3, 0, 0), Piece::White, Some(3)),
        ProductionCard::new(Funds::new(0, 0, 0, 0, 7, 0), Piece::Blue, Some(4)),
        ProductionCard::new(Funds::new(7, 0, 0, 3, 0, 0), Piece::Brown, Some(5)),
    ]
}

fn get_nobles() -> Vec<Noble> {
    vec![
        Noble::new(NobleId::new(1), Funds::new(0, 4, 4, 0, 0, 0)),
        Noble::new(NobleId::new(2), Funds::new(0, 0, 4, 0, 4, 0)),
        Noble::new(NobleId::new(3), Funds::new(4, 4, 0, 0, 0, 0)),
        Noble::new(NobleId::new(4), Funds::new(0, 0, 0, 4, 4, 0)),
        Noble::new(NobleId::new(5), Funds::new(3, 0, 0, 3, 3, 0)),
        Noble::new(NobleId::new(6), Funds::new(3, 3, 0, 3, 0, 0)),
        Noble::new(NobleId::new(7), Funds::new(3, 3, 3, 0, 0, 0)),
        Noble::new(NobleId::new(8), Funds::new(4, 0, 0, 4, 0, 0)),
        Noble::new(NobleId::new(9), Funds::new(0, 3, 3, 0, 3, 0)),
        Noble::new(NobleId::new(10), Funds::new(0, 0, 3, 3, 3, 0)),
    ]
}
