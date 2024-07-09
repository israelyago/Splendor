use std::collections::HashMap;
use std::slice::Iter;

use super::bank::CollectError;
use super::bank::Funds;
use super::noble::NobleId;
use super::player::Player;
use super::player::PlayerId;
use super::player::ReserveOperationFail;
use super::production_card::CardId;
use super::production_card::Identifiable;
use super::production_card::ProductionCard;

use super::bank;
use super::noble::Noble;
use super::piece::Piece;
use super::player;
use super::production_card;

const WINNING_POINTS_THRESHOLD: u8 = 15;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Winner {
    Winner(PlayerId),
    Draw(Vec<PlayerId>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoundType {
    Normal,
    LastRound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProductionTier {
    One,
    Two,
    Three,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ActionType {
    Normal,
    SelectNoble,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    PassTheTurn,
    CollectPieces(Vec<Piece>, Vec<Piece>),
    ReserveCardFromDeck(ProductionTier),
    ReserveCardFromBoard(production_card::CardId),
    BuyCard(production_card::CardId),
    SelectNoble(NobleId),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ActionFail {
    CannotReserveFromEmptyDeck,
    CardNotFoundOnBoard,
    NobleNotFound,
    InvalidBuyOperation(BuyOperationFail),
    InvalidReserve(ReserveOperationFail),
    InvalidCollect(CollectError),
    YouCannotSelectNobleNow,
    YouNeedToSelectNoble,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BuyOperationFail {
    NotEnoughFunds(Funds),
    CardNotFoundOnBoard,
}

#[derive(Debug, Clone)]
pub struct Board {
    players: Vec<player::Player>,
    player_turn: usize,
    pub bank: bank::Funds,
    decks: HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>>,
    cards_for_sale: HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>>,
    nobles: Vec<Noble>,
    action_needed: ActionType,
    round_type: RoundType,
    winner: Option<Winner>,
}

impl Board {
    pub fn new(
        players: Vec<player::Player>,
        bank: bank::Funds,
        decks: HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>>,
        nobles: Vec<Noble>,
    ) -> Self {
        let mut new_decks = decks;
        let mut cards_for_sale = HashMap::new();
        for (tier, prod_deck) in new_decks.iter_mut() {
            let mut to_sell: Vec<Identifiable<ProductionCard, CardId>> = vec![];
            for _ in 1..=4 {
                if let Some(to_add) = prod_deck.pop() {
                    to_sell.push(to_add);
                }
            }
            cards_for_sale.insert(*tier, to_sell);
        }
        Self {
            players,
            player_turn: 0,
            bank,
            decks: new_decks,
            cards_for_sale,
            nobles,
            action_needed: ActionType::Normal,
            round_type: RoundType::Normal,
            winner: None,
        }
    }

    pub fn get_deck(&self, tier: &ProductionTier) -> Vec<Identifiable<ProductionCard, CardId>> {
        self.decks.get(tier).unwrap().clone()
    }

    pub fn get_cards_for_sale(
        &self,
        tier: &ProductionTier,
    ) -> Vec<Identifiable<ProductionCard, CardId>> {
        self.cards_for_sale.get(tier).unwrap().clone()
    }

    pub fn get_nobles(&self) -> Vec<Noble> {
        self.nobles.clone()
    }

    fn get_winner(&self) -> Option<Winner> {
        let max_points_player = self
            .players
            .iter()
            .max_by_key(|p| p.total_victory_points())?;

        let max_points = max_points_player.total_victory_points();

        let possible_winners: Vec<&Player> = self
            .players
            .iter()
            .filter(|p| p.total_victory_points() == max_points)
            .collect();

        let player_with_least_amount_of_cards = possible_winners
            .clone()
            .into_iter()
            .min_by_key(|p| p.production_cards.len())?;

        let least_amount_of_cards = player_with_least_amount_of_cards.production_cards.len();

        let winners: Vec<PlayerId> = possible_winners
            .into_iter()
            .filter(|p| p.production_cards.len() == least_amount_of_cards)
            .map(|p| p.id.clone())
            .collect();

        if winners.len() == 1 {
            return Some(Winner::Winner(winners.get(0).unwrap().clone()));
        }

        Some(Winner::Draw(winners))
    }

    fn get_who_plays_next(&self) -> usize {
        let n_of_players = self.players.len();
        if self.player_turn == n_of_players - 1 {
            return 0;
        }
        self.player_turn + 1
    }

    pub fn get_players(&self) -> Iter<Player> {
        self.players.iter()
    }

    pub fn get_card_from_board(
        &self,
        card_id: &CardId,
    ) -> Option<Identifiable<ProductionCard, CardId>> {
        for cards in self.cards_for_sale.values() {
            for card in cards {
                if &card.uid == card_id {
                    return Some(card.clone());
                }
            }
        }
        None
    }

    pub fn get_who_is_playing_now(&self) -> &player::Player {
        self.players.get(self.player_turn).unwrap()
    }

    fn action_buy_production_card(&self, card_id: &CardId) -> Result<Board, ActionFail> {
        let mut new_board_state = self.clone();
        let card = self
            .get_card_from_board(card_id)
            .ok_or(ActionFail::InvalidBuyOperation(
                BuyOperationFail::CardNotFoundOnBoard,
            ))?;
        let mut player = self.players[self.player_turn].clone();

        let card_data = card.data.clone();

        let player_remaining_funds =
            production_card::ProductionCard::buy(player.clone(), card_data.clone())
                .map_err(ActionFail::InvalidBuyOperation)?;
        
        let used_coins = (player.funds - player_remaining_funds.clone()).expect("Player should have enough funds");
        player.funds = player_remaining_funds;
        player.production_cards.push(card);

        for (tier, cards) in &mut new_board_state.cards_for_sale {
            cards.retain(|c| &c.uid != card_id);
            let deck = new_board_state.decks.get_mut(tier).unwrap();
            if let Some(card_drawn) = deck.pop() {
                cards.push(card_drawn);
            }
        }
        new_board_state.players[self.player_turn] = player;

        new_board_state.bank = new_board_state.bank + used_coins;

        Ok(new_board_state)
    }

    fn action_collect_pieces(
        &self,
        collect_pieces: &[Piece],
        discard_pieces: &[Piece],
    ) -> Result<Board, ActionFail> {
        let current_player = self.get_who_is_playing_now();
        let player_funds = current_player.funds.clone();

        let collect_request = bank::CollectRequest::new(
            self.bank.clone(),
            player_funds,
            collect_pieces.to_vec(),
            discard_pieces.to_vec(),
        );
        let result = bank::Funds::collect(collect_request).map_err(ActionFail::InvalidCollect)?;

        let mut new_board_state = self.clone();
        new_board_state.bank = result.bank_funds;
        new_board_state.players[self.player_turn].funds = result.player_funds;

        Ok(new_board_state)
    }

    fn action_reserve_card_from_deck(&self, tier: &ProductionTier) -> Result<Board, ActionFail> {
        let mut new_board_state = self.clone();
        let deck = new_board_state.decks.get_mut(tier).unwrap();

        let card_drawn = deck.pop().ok_or(ActionFail::CannotReserveFromEmptyDeck)?;

        new_board_state.players[new_board_state.player_turn]
            .reserved_cards
            .push(card_drawn);
        Ok(new_board_state)
    }

    fn action_reserve_card(&self, card_id: &CardId) -> Result<Board, ActionFail> {
        self.reserve_card(card_id)
            .map_err(ActionFail::InvalidReserve)
    }

    fn reserve_card(&self, card_id: &CardId) -> Result<Board, ReserveOperationFail> {
        let success = player::Player::reserve_card(self, card_id)?;
        let mut new_board = self.clone();
        new_board.bank = success.bank_funds;
        new_board.players[new_board.player_turn] = success.player;
        Ok(new_board)
    }

    pub fn do_action(board: Board, action: &Action) -> Result<Board, ActionFail> {
        match board.action_needed {
            ActionType::Normal => {
                if let Action::SelectNoble(_) = action {
                    return Err(ActionFail::YouCannotSelectNobleNow);
                }
            }
            ActionType::SelectNoble => {
                if let Action::SelectNoble(_) = action {
                } else {
                    return Err(ActionFail::YouNeedToSelectNoble);
                }
            }
        }
        let mut new_board_state = board.clone();
        let mut has_selected_noble = false;
        match action {
            Action::PassTheTurn => {}
            Action::ReserveCardFromDeck(tier) => {
                new_board_state = board.action_reserve_card_from_deck(tier)?;
            }
            Action::CollectPieces(collect_pieces, discard_pieces) => {
                new_board_state = board.action_collect_pieces(collect_pieces, discard_pieces)?;
            }
            Action::ReserveCardFromBoard(card_id) => {
                new_board_state = board.action_reserve_card(card_id)?;
            }
            Action::BuyCard(card_id) => {
                new_board_state = board.action_buy_production_card(card_id)?;
            }
            Action::SelectNoble(noble_id) => {
                new_board_state.action_needed = ActionType::Normal;
                let nobles = new_board_state.nobles.clone();
                let noble = nobles
                    .iter()
                    .find(|noble| &noble.id == noble_id)
                    .ok_or(ActionFail::NobleNotFound)?;
                new_board_state.nobles.retain(|noble| &noble.id != noble_id);
                let current_player = &mut new_board_state.players[new_board_state.player_turn];
                current_player.nobles.push(noble.clone());
                has_selected_noble = true;
            }
        }

        let can_select_noble = new_board_state.can_select_noble() && !has_selected_noble;
        if can_select_noble {
            new_board_state.action_needed = ActionType::SelectNoble;
        }

        if new_board_state.has_some_player_passed_win_threshold() {
            new_board_state.round_type = RoundType::LastRound;
        }

        if !can_select_noble {
            if board.is_last_player_turn() && new_board_state.round_type == RoundType::LastRound {
                new_board_state.winner = board.get_winner();
            }

            new_board_state.player_turn = new_board_state.get_who_plays_next();
        }

        Ok(new_board_state)
    }

    fn can_select_noble(&self) -> bool {
        let player = self.get_who_is_playing_now();
        let player_produces = player
            .clone()
            .production_cards
            .into_iter()
            .map(|card| card.data.produces)
            .collect::<Vec<Piece>>();
        let player_produces_as_funds = &Funds::new_from_list(player_produces);

        for noble in &self.nobles {
            if (player_produces_as_funds.clone() - noble.cost.clone()).is_ok() {
                return true;
            }
        }

        false
    }

    fn has_some_player_passed_win_threshold(&self) -> bool {
        for p in &self.players {
            if p.total_victory_points() >= WINNING_POINTS_THRESHOLD {
                return true;
            }
        }
        false
    }

    fn is_last_player_turn(&self) -> bool {
        self.player_turn == self.players.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use bank::Funds;
    use player::{Player, PlayerId};
    use production_card::CardId;

    use super::*;

    fn get_default_production_card_cost() -> Funds {
        bank::Funds::new(1, 1, 0, 0, 0, 0)
    }

    fn get_production_card(card_id: CardId) -> Identifiable<ProductionCard, CardId> {
        let cost = get_default_production_card_cost();
        let card = production_card::ProductionCard::new(cost, Piece::Red, Some(1));
        Identifiable::new(card, card_id)
    }

    fn get_initial_player(id: PlayerId) -> player::Player {
        let funds = bank::Funds::new(0, 0, 0, 0, 0, 0);
        player::Player::new(id, funds, vec![], vec![])
    }

    fn get_player_with_winning_points(id: PlayerId, card_id: CardId, winning_points: u8) -> Player {
        let player = get_initial_player(id);
        let card = get_production_card(card_id.clone());
        let card = Identifiable::new(
            ProductionCard {
                victory_points: Some(winning_points),
                ..card.data
            },
            card_id,
        );
        Player {
            production_cards: vec![card],
            ..player
        }
    }

    fn get_initial_bank() -> bank::Funds {
        bank::Funds::new(7, 7, 7, 7, 7, 5)
    }

    fn get_default_board() -> Board {
        let p1 = get_initial_player(PlayerId::new(1));
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let bank_funds = get_initial_bank();
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
        let nobles = vec![];
        Board::new(vec![p1, p2, p3], bank_funds, decks, nobles)
    }

    #[test]
    fn auto_draw_necessary_cards() {
        let board = get_default_board();
        assert_eq!(
            board
                .cards_for_sale
                .get(&ProductionTier::One)
                .unwrap()
                .len(),
            4
        );
        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 1);

        let p1 = get_initial_player(PlayerId::new(1));
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let bank_funds = get_initial_bank();
        let decks = HashMap::from([(
            ProductionTier::One,
            vec![
                get_production_card(CardId::new(5)),
                get_production_card(CardId::new(4)),
            ],
        )]);

        let board = Board::new(vec![p1, p2, p3], bank_funds, decks, vec![]);
        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 0);
        assert_eq!(
            board
                .cards_for_sale
                .get(&ProductionTier::One)
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn can_pass_the_turn() {
        let board = get_default_board();
        let action = &Action::PassTheTurn;

        let current_player = 0;

        let result = Board::do_action(board, action);
        assert!(result.is_ok());

        let board_after = result.unwrap();
        assert_ne!(current_player, board_after.player_turn);
    }

    #[test]
    fn turn_cycles_back_to_first_person() {
        let board = get_default_board();
        let action = &Action::PassTheTurn;

        let result = Board::do_action(board, action);
        assert!(result.is_ok());

        let board = result.unwrap();
        assert_eq!(1, board.player_turn);

        let result = Board::do_action(board, action);
        assert!(result.is_ok());

        let board = result.unwrap();
        assert_eq!(2, board.player_turn);

        // Cycle back to first player
        let result = Board::do_action(board, action);
        assert!(result.is_ok());

        let board = result.unwrap();
        assert_eq!(0, board.player_turn);
    }

    #[test]
    fn can_collect_up_to_three_pieces() {
        let board = get_default_board();
        let action = &Action::CollectPieces(vec![Piece::Red, Piece::Blue, Piece::White], vec![]);

        let result = Board::do_action(board, action);
        assert!(result.is_ok());

        let result = result.unwrap();
        let red = result.bank.funds[&Piece::Red];
        let blue = result.bank.funds[&Piece::Blue];
        let white = result.bank.funds[&Piece::White];
        assert_eq!(red, 6);
        assert_eq!(blue, 6);
        assert_eq!(white, 6);

        let expected_player_funds = bank::Funds::new(1, 0, 1, 0, 1, 0);
        let first_player = result.players.get(0);
        assert_eq!(first_player.unwrap().funds, expected_player_funds);

        let board = get_default_board();
        let action = &Action::CollectPieces(
            vec![Piece::Red, Piece::Blue, Piece::White, Piece::Red],
            vec![],
        );

        let result = Board::do_action(board, action).unwrap_err();
        assert_eq!(
            result,
            ActionFail::InvalidCollect(CollectError::CannotCollectMoreThanThree)
        );
    }

    #[test]
    fn can_reserve_card_from_deck() {
        let board = get_default_board();

        let first_player = board.players.get(0).unwrap();

        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 1);
        assert_eq!(first_player.reserved_cards.len(), 0);
        assert_eq!(board.player_turn, 0);

        let action = &Action::ReserveCardFromDeck(ProductionTier::One);

        let board = Board::do_action(board, action).unwrap();

        // Assert that p1 has a production card reserved
        let first_player = board.players.get(0).unwrap();

        assert_eq!(first_player.reserved_cards.len(), 1);

        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 0);

        // Make sure the turn is passed to the next player
        assert_eq!(board.player_turn, 1);
        assert_eq!(board.round_type, RoundType::Normal);

        // Assert that we cannot reserve from an empty deck
        let action = &Action::ReserveCardFromDeck(ProductionTier::One);
        let result = Board::do_action(board.clone(), action).unwrap_err();
        assert_eq!(result, ActionFail::CannotReserveFromEmptyDeck);
    }

    #[test]
    fn can_reserve_card_from_board() {
        let board = get_default_board();

        let first_player = board.players.get(0).unwrap();

        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 1);
        assert_eq!(first_player.reserved_cards.len(), 0);
        assert_eq!(first_player.funds.funds.get(&Piece::Golden).unwrap(), &0);
        assert_eq!(board.player_turn, 0);

        let action = &Action::ReserveCardFromBoard(CardId::new(1));

        let result = Board::do_action(board, action).unwrap();

        let player_one = result.players.get(0).unwrap();

        assert_eq!(player_one.reserved_cards.len(), 1);
        assert_eq!(player_one.funds.funds.get(&Piece::Golden).unwrap(), &1);
        assert_eq!(result.player_turn, 1);

        let player_reserved_card = player_one.reserved_cards.get(0).unwrap();

        let expected_card = get_production_card(CardId::new(1));

        assert_eq!(player_reserved_card.uid, expected_card.uid);
        assert_eq!(player_reserved_card.data, expected_card.data);
    }

    #[test]
    fn cannot_reserve_card_that_does_not_exist() {
        let board = get_default_board();
        let action = &Action::ReserveCardFromBoard(CardId::new(255));
        let result = Board::do_action(board, action).unwrap_err();
        assert_eq!(
            result,
            ActionFail::InvalidReserve(ReserveOperationFail::CardNotFound)
        );
    }

    #[test]
    fn can_buy_production_card() {
        let board = get_default_board();
        let action = &Action::CollectPieces(vec![Piece::Red, Piece::Green, Piece::Blue], vec![]);
        let action_pass = &Action::PassTheTurn;

        assert_eq!(board.bank, Funds::new(7, 7, 7, 7, 7, 5));

        let board = Board::do_action(board, action).unwrap();
        assert_eq!(board.bank, Funds::new(6, 6, 6, 7, 7, 5));
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        let board = Board::do_action(board, action).unwrap();
        assert_eq!(board.bank, Funds::new(5, 5, 5, 7, 7, 5));

        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 0);
        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 1);
        assert_eq!(
            board
                .cards_for_sale
                .get(&ProductionTier::One)
                .unwrap()
                .len(),
            4
        );
        let expected_player_funds = bank::Funds::new(2, 2, 2, 0, 0, 0);
        assert_eq!(player_one.funds, expected_player_funds);

        let action = &Action::BuyCard(CardId::new(1));
        let board = Board::do_action(board, action).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 1);

        // Make sure that the money was transfered, from the player to the bank
        let expected_player_funds = bank::Funds::new(1, 1, 2, 0, 0, 0);
        let expected_bank_funds = bank::Funds::new(6, 6, 5, 7, 7, 5);
        assert_eq!(player_one.funds, expected_player_funds);
        assert_eq!(board.bank, expected_bank_funds);

        // Makes sure a new card was drawn
        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 0);
        assert_eq!(
            board
                .cards_for_sale
                .get(&ProductionTier::One)
                .unwrap()
                .len(),
            4
        );

        // Makes sure the card is not on sale anymore
        let action_fail = Board::do_action(board.clone(), action).unwrap_err();
        assert_eq!(
            action_fail,
            ActionFail::InvalidBuyOperation(BuyOperationFail::CardNotFoundOnBoard)
        );

        // Make sure you used the card instead of the coins
        // Using 1 production card (red), the player red coin should is not reduced, only 1 green

        let action = &Action::BuyCard(CardId::new(2));
        let board = Board::do_action(board, action).unwrap();

        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 2);
        
        let expected_player_funds = bank::Funds::new(1, 0, 2, 0, 0, 0);
        let expected_bank_funds = bank::Funds::new(6, 7, 5, 7, 7, 5);
        assert_eq!(player_one.funds, expected_player_funds);
        assert_eq!(board.bank, expected_bank_funds);

    }

    #[test]
    fn cannot_buy_production_card_if_there_is_no_pieces() {
        let board = get_default_board();
        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 0);

        let action = &Action::BuyCard(CardId::new(1));

        let result = Board::do_action(board, action).unwrap_err();
        let expected_funds = bank::Funds::new(1, 1, 0, 0, 0, 0);
        assert_eq!(
            result,
            ActionFail::InvalidBuyOperation(BuyOperationFail::NotEnoughFunds(expected_funds))
        );
    }

    #[test]
    fn can_buy_card_using_golden_and_cards_production() {
        let funds = bank::Funds::new(1, 1, 0, 0, 0, 2);
        let red_production = production_card::ProductionCard::new(
            get_default_production_card_cost(),
            Piece::Red,
            Some(1),
        );
        let red_card = Identifiable::new(red_production, CardId::new(100));
        let p1 = player::Player::new(PlayerId::new(1), funds, vec![red_card], vec![]);
        let p2 = get_initial_player(PlayerId::new(2));
        let p3 = get_initial_player(PlayerId::new(3));
        let bank_funds = get_initial_bank();

        let blue_production = production_card::ProductionCard::new(
            bank::Funds::new(3, 1, 0, 0, 0, 0),
            Piece::Blue,
            Some(1),
        );
        let card_to_buy = Identifiable::new(blue_production, CardId::new(101));

        let decks = HashMap::from([(
            ProductionTier::One,
            vec![
                get_production_card(CardId::new(5)),
                get_production_card(CardId::new(4)),
                get_production_card(CardId::new(3)),
                get_production_card(CardId::new(2)),
                get_production_card(CardId::new(1)),
                card_to_buy,
            ],
        )]);

        let board = Board::new(vec![p1, p2, p3], bank_funds, decks, vec![]);

        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 1);

        let action = &Action::BuyCard(CardId::new(101));
        let board = Board::do_action(board, action).unwrap();

        let player_one = board.players.get(0).unwrap();
        let player_expected_funds = Funds::new(0, 0, 0, 0, 0, 1);
        assert_eq!(player_one.production_cards.len(), 2);
        assert_eq!(player_one.funds, player_expected_funds);
    }

    #[test]
    fn can_select_a_noble_only_after_buying() {
        let board = get_default_board();

        let noble_to_select = Noble {
            id: NobleId::new(1),
            cost: bank::Funds::new(1, 0, 0, 0, 0, 0),
        };

        let second_noble = Noble {
            id: NobleId::new(2),
            cost: bank::Funds::new(1, 0, 0, 0, 0, 0),
        };

        let board = Board {
            nobles: vec![noble_to_select.clone(), second_noble.clone()],
            ..board
        };

        // Make sure you can't select noble at any time
        let action = &Action::SelectNoble(NobleId::new(1));
        let action_fail = Board::do_action(board.clone(), action).unwrap_err();
        assert_eq!(action_fail, ActionFail::YouCannotSelectNobleNow);

        let action = &Action::CollectPieces(vec![Piece::Red, Piece::Green, Piece::Blue], vec![]);
        let action_pass = &Action::PassTheTurn;

        let board = Board::do_action(board, action).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.production_cards.len(), 0);
        assert_eq!(board.decks.get(&ProductionTier::One).unwrap().len(), 1);
        assert_eq!(
            board
                .cards_for_sale
                .get(&ProductionTier::One)
                .unwrap()
                .len(),
            4
        );

        let action = &Action::BuyCard(CardId::new(1));

        assert_eq!(board.action_needed, ActionType::Normal);
        let board = Board::do_action(board, action).unwrap();
        // Assert that the player is still playing
        assert_eq!(board.player_turn, 0);
        // Assert that the action needed is special: To select a noble
        assert_eq!(board.action_needed, ActionType::SelectNoble);

        // And that we can't do other actions
        let action = &Action::CollectPieces(vec![Piece::Red, Piece::Green, Piece::Blue], vec![]);
        let action_fail = Board::do_action(board.clone(), action).unwrap_err();

        assert_eq!(action_fail, ActionFail::YouNeedToSelectNoble);

        // Make sure we still dont have any nobles
        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.nobles.len(), 0);

        // Make sure we can't select inexistent noble
        let action = &Action::SelectNoble(NobleId::new(255));
        let select_err = Board::do_action(board.clone(), action).unwrap_err();
        assert_eq!(select_err, ActionFail::NobleNotFound);

        // Assert that we can select noble
        let action = &Action::SelectNoble(NobleId::new(1));
        let board = Board::do_action(board, action).unwrap();
        
        let player_one = board.players.get(0).unwrap();
        assert_eq!(player_one.nobles.len(), 1);
        assert_eq!(player_one.nobles.get(0).unwrap(), &noble_to_select);

        // Assert that we cannot select noble again (even if the board would normally allow)
        assert_eq!(board.action_needed, ActionType::Normal);
        assert_eq!(board.player_turn, 1);

    }

    #[test]
    fn end_round_triggered_after_hitting_15_points() {
        let player_one = get_initial_player(PlayerId::new(1));
        let player_one = player::Player {
            funds: bank::Funds::new(3, 3, 2, 2, 0, 0),
            production_cards: vec![
                get_production_card(CardId::new(100)),
                get_production_card(CardId::new(101)),
                get_production_card(CardId::new(102)),
                get_production_card(CardId::new(103)),
                get_production_card(CardId::new(104)),
                get_production_card(CardId::new(105)),
                get_production_card(CardId::new(106)),
                get_production_card(CardId::new(107)),
                get_production_card(CardId::new(108)),
                get_production_card(CardId::new(109)),
                get_production_card(CardId::new(110)),
                get_production_card(CardId::new(111)),
            ],
            ..player_one
        };
        let cost = bank::Funds::new(1, 1, 1, 0, 0, 0);
        let card_to_buy = production_card::ProductionCard::new(cost, Piece::Blue, Some(3));
        let card_to_buy = Identifiable::new(card_to_buy, CardId::new(112));

        let mut cards_for_sale: HashMap<ProductionTier, Vec<Identifiable<ProductionCard, CardId>>> =
            HashMap::new();
        cards_for_sale.insert(ProductionTier::One, vec![card_to_buy]);
        let board = get_default_board();
        let board = Board {
            players: vec![player_one, get_initial_player(PlayerId::new(2))],
            cards_for_sale,
            ..board
        };

        // assert that the EndGame was not triggered (to be created)
        assert_eq!(board.round_type, RoundType::Normal);

        // Buy production card 112
        let action = Action::BuyCard(CardId::new(112));
        let board = Board::do_action(board, &action).unwrap();

        // assert that the EndGame was triggered
        assert_eq!(board.round_type, RoundType::LastRound);
        assert_eq!(board.winner, None);

        // Pass the 2nd player turn, ending the game
        let action = Action::PassTheTurn;
        let board = Board::do_action(board, &action).unwrap();
        assert_eq!(board.winner.unwrap(), Winner::Winner(PlayerId::new(1)));
    }

    #[test]
    fn correctly_get_winner() {
        let board = get_default_board();
        let player_one = get_player_with_winning_points(PlayerId::new(1), CardId::new(100), 16);
        let player_two = get_player_with_winning_points(PlayerId::new(2), CardId::new(101), 10);
        let player_three = get_player_with_winning_points(PlayerId::new(3), CardId::new(102), 17);
        let board = Board {
            players: vec![player_one, player_two, player_three.clone()],
            round_type: RoundType::LastRound,
            ..board
        };
        let action_pass = &Action::PassTheTurn;

        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        assert_eq!(board.winner.unwrap(), Winner::Winner(player_three.id));
    }

    #[test]
    fn correctly_get_winner_on_draw_in_points() {
        let board = get_default_board();

        let player_one = get_player_with_winning_points(PlayerId::new(2), CardId::new(101), 16);
        let player_two = get_player_with_winning_points(PlayerId::new(1), CardId::new(100), 15);
        let mut player_two_cards = player_two.production_cards.clone();
        player_two_cards.push(get_production_card(CardId::new(103)));

        // Player two now has 16, making it a draw with player one, but also has 2 production cards
        let player_two = Player {
            production_cards: player_two_cards,
            ..player_two
        };
        let player_three = get_player_with_winning_points(PlayerId::new(3), CardId::new(102), 15);
        let board = Board {
            players: vec![player_one.clone(), player_two, player_three],
            round_type: RoundType::LastRound,
            ..board
        };
        let action_pass = &Action::PassTheTurn;

        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        // player one wins because he has the least amount of cards (game rule)
        assert_eq!(board.winner.unwrap(), Winner::Winner(player_one.id));
    }

    #[test]
    fn correctly_get_winners_on_draw() {
        // This is *not* described in the original game rules: What if there is a draw in points, and amount of cards?
        // We assume that the players got a draw.

        let board = get_default_board();
        let player_one = get_player_with_winning_points(PlayerId::new(1), CardId::new(100), 16);
        let player_two = get_player_with_winning_points(PlayerId::new(2), CardId::new(101), 17);
        let player_three = get_player_with_winning_points(PlayerId::new(3), CardId::new(102), 17);
        let board = Board {
            players: vec![player_one, player_two.clone(), player_three.clone()],
            round_type: RoundType::LastRound,
            ..board
        };
        let action_pass = &Action::PassTheTurn;

        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();
        let board = Board::do_action(board, action_pass).unwrap();

        assert_eq!(
            board.winner.unwrap(),
            Winner::Draw(vec![player_two.id, player_three.id])
        );
    }
}
