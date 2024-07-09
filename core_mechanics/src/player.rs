use std::collections::HashMap;

use super::bank;
use super::board;
use super::noble::Noble;
use super::noble::NOBLE_VICTORY_POINTS;
use super::piece::Piece;
use super::production_card;
use super::production_card::CardId;
use super::production_card::Identifiable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerId {
    id: u8,
}

impl PlayerId {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub id: PlayerId,
    pub funds: bank::Funds,
    pub production_cards: Vec<Identifiable<production_card::ProductionCard, CardId>>,
    pub reserved_cards: Vec<Identifiable<production_card::ProductionCard, CardId>>,
    pub nobles: Vec<Noble>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReserveOperationFail {
    NotEnoughPieces(Vec<(Piece, u8)>),
    MaximumReservedCardsExceed,
    CardNotFound,
}

#[derive(Debug)]
pub struct ReserveOperationSuccess {
    pub bank_funds: bank::Funds,
    pub player: Player,
}

impl ReserveOperationSuccess {
    pub fn new(bank_funds: bank::Funds, player: Player) -> Self {
        Self { bank_funds, player }
    }
}

impl Player {
    pub fn new(
        id: PlayerId,
        funds: bank::Funds,
        production_cards: Vec<Identifiable<production_card::ProductionCard, CardId>>,
        reserved_cards: Vec<Identifiable<production_card::ProductionCard, CardId>>,
    ) -> Self {
        Self {
            id,
            funds,
            production_cards,
            reserved_cards,
            nobles: vec![],
        }
    }

    pub fn get_production(&self) -> bank::Funds {
        Self::get_funds_from_production_cards(self.production_cards.clone())
    }

    pub fn get_funds_from_production_cards(
        production_cards: Vec<Identifiable<production_card::ProductionCard, CardId>>,
    ) -> bank::Funds {
        let mut funds = HashMap::from([
            (Piece::Red, 0),
            (Piece::Green, 0),
            (Piece::Blue, 0),
            (Piece::Brown, 0),
            (Piece::White, 0),
            (Piece::Golden, 0),
        ]);

        for card in production_cards {
            let card = card.data;
            let current_funds = funds.clone();
            let current_amount = current_funds.get(&card.produces).unwrap();
            funds.insert(card.produces, *current_amount + 1);
        }

        bank::Funds::new_from(funds)
    }

    pub fn reserve_card(
        board: &board::Board,
        card_id: &CardId,
    ) -> Result<ReserveOperationSuccess, ReserveOperationFail> {
        let card = board::Board::get_card_from_board(board, card_id)
            .ok_or(ReserveOperationFail::CardNotFound)?;

        let player = board.get_who_is_playing_now();

        if player.reserved_cards.len() >= 3 {
            return Result::Err(ReserveOperationFail::MaximumReservedCardsExceed);
        }

        let bank_funds = &board.bank;

        let mut bank = bank_funds.funds.clone();
        let mut player_updated = player.clone();
        let bank_golden_pieces = bank_funds.funds.get(&Piece::Golden).unwrap_or(&0);
        player_updated.reserved_cards.push(card);

        if *bank_golden_pieces > 0 {
            bank.insert(Piece::Golden, bank_golden_pieces - 1);
            let player_golden_quantity = *player.funds.funds.get(&Piece::Golden).unwrap_or(&0);
            player_updated
                .funds
                .funds
                .insert(Piece::Golden, player_golden_quantity + 1);
        }

        Result::Ok(ReserveOperationSuccess::new(
            bank::Funds::new_from(bank),
            player_updated,
        ))
    }

    pub fn total_victory_points(&self) -> u8 {
        let mut total_points = 0;
        for p in &self.production_cards {
            if let Some(points) = p.data.victory_points {
                total_points += points;
            }
        }

        for _ in &self.nobles {
            total_points += NOBLE_VICTORY_POINTS;
        }

        total_points
    }
}

#[cfg(test)]
mod tests {

    use board::{Board, ProductionTier};
    use production_card::ProductionCard;

    use super::*;

    fn get_production_card(card_id: CardId) -> Identifiable<ProductionCard, CardId> {
        let cost = get_initial_bank();
        let card = production_card::ProductionCard::new(cost, Piece::Red, Some(1));
        Identifiable::new(card, card_id)
    }

    fn get_initial_player(id: PlayerId) -> Player {
        let funds = bank::Funds::new(0, 0, 0, 0, 0, 0);
        Player::new(id, funds, vec![], vec![])
    }

    fn get_initial_bank() -> bank::Funds {
        bank::Funds::new(7, 7, 7, 7, 7, 5)
    }

    fn get_default_cost() -> bank::Funds {
        bank::Funds::new(0, 1, 2, 1, 1, 0)
    }

    #[test]
    fn can_reserve_card() {
        let player_funds = bank::Funds::new(3, 2, 3, 1, 1, 1);

        let p1 = Player::new(PlayerId::new(1), player_funds, vec![], vec![]);
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let bank_funds = bank::Funds::new(8, 8, 8, 8, 8, 8);
        let decks = HashMap::from([(
            ProductionTier::One,
            vec![
                get_production_card(CardId::new(5)),
                get_production_card(CardId::new(4)),
                get_production_card(CardId::new(3)),
                get_production_card(CardId::new(2)),
                get_production_card(CardId::new(1)),
            ],
        )]);
        let board = Board::new(vec![p1, p2, p3], bank_funds, decks, vec![]);

        let result = Player::reserve_card(&board, &CardId::new(1)).unwrap();

        let expected_bank_funds = bank::Funds::new(8, 8, 8, 8, 8, 7);
        assert_eq!(result.bank_funds, expected_bank_funds);

        let expected_player_funds = bank::Funds::new(3, 2, 3, 1, 1, 2);
        assert_eq!(result.player.funds, expected_player_funds);

        assert_eq!(result.player.reserved_cards.len(), 1);
        let player_prod_card = result.player.reserved_cards.get(0).unwrap();
        let expected_prod_card = get_production_card(CardId::new(1));
        assert_eq!(player_prod_card.uid, expected_prod_card.uid);
        assert_eq!(player_prod_card.data, expected_prod_card.data);
    }

    #[test]
    fn cannot_reserve_more_than_3() {
        let prod_card = production_card::ProductionCard::new(get_default_cost(), Piece::Red, None);

        let bank_funds = bank::Funds::new(8, 8, 8, 8, 8, 8);
        let player_funds = bank::Funds::new(3, 2, 3, 1, 1, 1);
        let p1 = Player::new(
            PlayerId::new(1),
            player_funds,
            vec![],
            vec![
                Identifiable::new(prod_card.clone(), CardId::new(6)),
                Identifiable::new(prod_card.clone(), CardId::new(7)),
                Identifiable::new(prod_card, CardId::new(8)),
            ],
        );
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let decks = HashMap::from([(
            ProductionTier::One,
            vec![
                get_production_card(CardId::new(5)),
                get_production_card(CardId::new(4)),
                get_production_card(CardId::new(3)),
                get_production_card(CardId::new(2)),
                get_production_card(CardId::new(1)),
            ],
        )]);
        let board = Board::new(vec![p1, p2, p3], bank_funds, decks, vec![]);

        let result = Player::reserve_card(&board, &CardId::new(1));
        assert!(result.is_err());
        let result = result.unwrap_err();
        assert_eq!(result, ReserveOperationFail::MaximumReservedCardsExceed);
    }

    #[test]
    fn do_not_get_golden_if_there_is_none() {
        let bank_funds = bank::Funds::new(8, 8, 8, 8, 8, 0);
        let player_funds = bank::Funds::new(3, 2, 3, 1, 1, 1);

        let p1 = Player::new(PlayerId::new(1), player_funds, vec![], vec![]);
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let decks = HashMap::from([(
            ProductionTier::One,
            vec![
                get_production_card(CardId::new(5)),
                get_production_card(CardId::new(4)),
                get_production_card(CardId::new(3)),
                get_production_card(CardId::new(2)),
                get_production_card(CardId::new(1)),
            ],
        )]);
        let board = Board::new(vec![p1, p2, p3], bank_funds, decks, vec![]);

        let result = Player::reserve_card(&board, &CardId::new(1));
        assert!(result.is_ok());

        let expected_bank_funds = bank::Funds::new(8, 8, 8, 8, 8, 0);
        let result = result.unwrap();
        assert_eq!(result.bank_funds, expected_bank_funds);

        let expected_player_funds = bank::Funds::new(3, 2, 3, 1, 1, 1);
        assert_eq!(result.player.funds, expected_player_funds);

        assert_eq!(result.player.reserved_cards.len(), 1);
        let player_prod_card = result.player.reserved_cards.get(0).unwrap();
        let expected_prod_card = get_production_card(CardId::new(1));
        assert_eq!(player_prod_card.uid, expected_prod_card.uid);
        assert_eq!(player_prod_card.data, expected_prod_card.data);
    }
}
