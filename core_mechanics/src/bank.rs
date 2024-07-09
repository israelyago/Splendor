use std::{collections::HashMap, ops::Add, ops::Sub};

use super::bank;
use super::piece::Piece;

const MIN_PILE_SIZE_TO_COLLECT_TWO_EQUALS: u8 = 4;

#[derive(Debug, PartialEq, Eq)]
pub enum CollectError {
    CollectedGolden,
    Collected2OfTheSameWithAnother,
    NotEnoughAtTheBank(Piece),
    CannotCollect2WhenResourceIsAlmostEmpty,
    CannotCollectMoreThanThree,
    CannotDiscardMoreThanThree,
    NotEnoughPiecesToDiscard,
    CannotStoreMoreThan10,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CollectSuccess {
    pub bank_funds: bank::Funds,
    pub player_funds: bank::Funds,
}

impl CollectSuccess {
    fn new(bank_funds: bank::Funds, player_funds: bank::Funds) -> Self {
        Self {
            bank_funds,
            player_funds,
        }
    }
}
#[derive(Debug, Clone)]
pub struct CollectRequest {
    bank_funds: bank::Funds,
    player_funds: bank::Funds,
    want_to_collect: Vec<Piece>,
    discard: Vec<Piece>,
}

impl CollectRequest {
    pub fn new(
        bank_funds: Funds,
        player_funds: Funds,
        want_to_collect: Vec<Piece>,
        discard: Vec<Piece>,
    ) -> Self {
        Self {
            bank_funds,
            player_funds,
            want_to_collect,
            discard,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Funds {
    pub funds: HashMap<Piece, u8>,
}

impl Funds {
    pub fn new(red: u8, green: u8, blue: u8, brown: u8, white: u8, golden: u8) -> Self {
        Self {
            funds: HashMap::from([
                (Piece::Red, red),
                (Piece::Green, green),
                (Piece::Blue, blue),
                (Piece::Brown, brown),
                (Piece::White, white),
                (Piece::Golden, golden),
            ]),
        }
    }

    pub fn new_from(funds: HashMap<Piece, u8>) -> Self {
        Self {
            funds: HashMap::from([
                (Piece::Red, *funds.get(&Piece::Red).unwrap_or(&0)),
                (Piece::Green, *funds.get(&Piece::Green).unwrap_or(&0)),
                (Piece::Blue, *funds.get(&Piece::Blue).unwrap_or(&0)),
                (Piece::Brown, *funds.get(&Piece::Brown).unwrap_or(&0)),
                (Piece::White, *funds.get(&Piece::White).unwrap_or(&0)),
                (Piece::Golden, *funds.get(&Piece::Golden).unwrap_or(&0)),
            ]),
        }
    }

    pub fn new_from_list(funds: Vec<Piece>) -> Self {
        let mut new_funds: HashMap<Piece, u8> = HashMap::from([
            (Piece::Red, 0),
            (Piece::Green, 0),
            (Piece::Blue, 0),
            (Piece::Brown, 0),
            (Piece::White, 0),
            (Piece::Golden, 0),
        ]);
        for p in funds {
            new_funds.insert(p, *new_funds.get(&p).unwrap_or(&0) + 1);
        }
        Self { funds: new_funds }
    }

    pub fn collect(collect_request: CollectRequest) -> Result<CollectSuccess, CollectError> {
        for p in &collect_request.want_to_collect {
            if p == &Piece::Golden {
                return Err(CollectError::CollectedGolden);
            }
        }
        let total_amount_of_pieces = collect_request.want_to_collect.len();
        if total_amount_of_pieces > 3 {
            return Err(CollectError::CannotCollectMoreThanThree);
        }

        if collect_request.discard.len() > 3 {
            return Err(CollectError::CannotDiscardMoreThanThree);
        }

        let player_request_as_funds = Funds::new_from_list(collect_request.want_to_collect.clone());
        for (piece, q) in &player_request_as_funds.funds {
            if q >= &2 && total_amount_of_pieces == 3 {
                return Err(CollectError::Collected2OfTheSameWithAnother);
            }
            let current_quantity_in_bank =
                collect_request.bank_funds.funds.get(piece).unwrap_or(&0);
            if q >= &2 && current_quantity_in_bank < &MIN_PILE_SIZE_TO_COLLECT_TWO_EQUALS {
                return Err(CollectError::CannotCollect2WhenResourceIsAlmostEmpty);
            }
        }

        let discard_cards_as_funds = Funds::new_from_list(collect_request.discard);

        let result_bank_funds = (collect_request.bank_funds + discard_cards_as_funds.clone())
            - player_request_as_funds.clone();

        if let Err(subtraction_error) = result_bank_funds {
            return match subtraction_error {
                FundsSubtractionError::NotEnoughFunds(piece_missing) => {
                    Err(CollectError::NotEnoughAtTheBank(piece_missing))
                }
            };
        }

        let result_bank_funds = result_bank_funds.unwrap();
        let new_player_funds = ((collect_request.player_funds + player_request_as_funds)
            - discard_cards_as_funds)
            .map_err(|err| match err {
                FundsSubtractionError::NotEnoughFunds(_) => CollectError::NotEnoughPiecesToDiscard,
            })?;

        let total_amount_of_pieces = new_player_funds
            .funds
            .iter()
            .map(|fund| *fund.1)
            .sum::<u8>();
        if total_amount_of_pieces > 10 {
            return Err(CollectError::CannotStoreMoreThan10);
        }

        Ok(CollectSuccess::new(result_bank_funds, new_player_funds))
    }
}

impl From<Funds> for Vec<Piece> {
    fn from(funds: Funds) -> Self {
        let mut pieces: Vec<Piece> = vec![];

        let all_pieces = vec![
            Piece::Blue,
            Piece::Brown,
            Piece::Golden,
            Piece::Green,
            Piece::Red,
            Piece::White,
        ];

        for color in all_pieces {
            let quantity = *funds.funds.get(&color).unwrap();
            for _ in 0..quantity {
                pieces.push(color)
            }
        }

        pieces
    }
}

#[derive(Debug)]
pub enum FundsSubtractionError {
    NotEnoughFunds(Piece),
}

impl Sub<Funds> for Funds {
    type Output = Result<Self, FundsSubtractionError>;

    fn sub(self, rhs: Funds) -> Self::Output {
        let mut funds_remaining = self.clone();

        for cost_piece in rhs.funds {
            let (piece, quantity) = cost_piece;
            if !self.funds.contains_key(&piece) {
                continue;
            }
            let current_amount = match self.funds.get(&piece) {
                None => 0,
                Some(i) => *i,
            };
            if quantity > current_amount {
                return Err(FundsSubtractionError::NotEnoughFunds(piece));
            }
            funds_remaining
                .funds
                .insert(piece, current_amount - quantity);
        }

        Ok(funds_remaining)
    }
}

impl Add<Funds> for Funds {
    type Output = Funds;

    fn add(self, rhs: Funds) -> Self::Output {
        let mut new_funds = self.clone();

        for (piece, quantity) in rhs.funds {
            let current_amount = match self.funds.get(&piece) {
                None => 0,
                Some(i) => *i,
            };
            new_funds.funds.insert(piece, current_amount + quantity);
        }

        new_funds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_default_funds() -> Funds {
        Funds::new(8, 8, 8, 8, 8, 8)
    }

    #[test]
    fn can_collect_3_pieces() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Red, Piece::White],
            vec![],
        );
        let response = Funds::collect(collect_request).unwrap();

        let expected_bank_funds = Funds::new(7, 8, 7, 8, 7, 8);
        assert_eq!(response.bank_funds, expected_bank_funds);

        let expected_player_funds = Funds::new(1, 0, 1, 0, 1, 0);
        assert_eq!(response.player_funds, expected_player_funds);
    }

    #[test]
    fn cannot_collect_golden() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Red, Piece::Golden],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::CollectedGolden))
    }

    #[test]
    fn cannot_store_more_than_10_pieces() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(2, 2, 2, 2, 0, 0),
            vec![Piece::Red, Piece::Green, Piece::Blue],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::CannotStoreMoreThan10))
    }

    #[test]
    fn cannot_collect_2_of_the_same_with_another_one() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Blue, Piece::Red],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::Collected2OfTheSameWithAnother));

        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Blue, Piece::Blue],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::Collected2OfTheSameWithAnother));
    }

    #[test]
    fn cannot_collect_more_than_three() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Red, Piece::Green, Piece::Blue, Piece::White],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::CannotCollectMoreThanThree));
    }

    #[test]
    fn can_collect_only_2_of_the_same() {
        let collect_request = CollectRequest::new(
            get_default_funds(),
            Funds::new(1, 1, 1, 1, 1, 1),
            vec![Piece::Blue, Piece::Blue],
            vec![],
        );
        let response = Funds::collect(collect_request).unwrap();
        let expected_player_funds = Funds::new(1, 1, 3, 1, 1, 1);
        assert_eq!(response.player_funds, expected_player_funds);
    }

    #[test]
    fn cannot_collect_when_there_is_not_enough_at_the_bank() {
        let collect_request = CollectRequest::new(
            Funds::new(1, 1, 0, 1, 1, 1),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Red],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(response, Err(CollectError::NotEnoughAtTheBank(Piece::Blue)));
    }

    #[test]
    fn cannot_collect_2_of_the_same_when_the_stack_is_almost_empty() {
        let collect_request = CollectRequest::new(
            Funds::new(1, 1, MIN_PILE_SIZE_TO_COLLECT_TWO_EQUALS - 1, 1, 1, 1),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Blue],
            vec![],
        );
        let response = Funds::collect(collect_request);
        assert_eq!(
            response,
            Err(CollectError::CannotCollect2WhenResourceIsAlmostEmpty)
        );

        let collect_request = CollectRequest::new(
            Funds::new(1, 1, MIN_PILE_SIZE_TO_COLLECT_TWO_EQUALS, 1, 1, 1),
            Funds::new(0, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Blue],
            vec![],
        );
        let response = Funds::collect(collect_request).unwrap();
        let expected_player_funds = Funds::new(0, 0, 2, 0, 0, 0);
        assert_eq!(response.player_funds, expected_player_funds);

        let expected_bank_funds =
            Funds::new(1, 1, MIN_PILE_SIZE_TO_COLLECT_TWO_EQUALS - 2, 1, 1, 1);
        assert_eq!(response.bank_funds, expected_bank_funds);
    }

    #[test]
    fn can_discard_to_collect_new_pieces() {
        let collect_request = CollectRequest::new(
            Funds::new(1, 1, 1, 1, 1, 1),
            Funds::new(2, 2, 2, 2, 2, 0),
            vec![Piece::Blue, Piece::Red, Piece::White],
            vec![Piece::Brown, Piece::Green, Piece::Green],
        );
        let response = Funds::collect(collect_request).unwrap();

        let expected_bank_funds = Funds::new(0, 3, 0, 2, 0, 1);
        assert_eq!(response.bank_funds, expected_bank_funds);

        let expected_player_funds = Funds::new(3, 0, 3, 1, 3, 0);
        assert_eq!(response.player_funds, expected_player_funds);
    }

    #[test]
    fn cannot_discard_if_the_player_do_not_have_the_pieces() {
        let collect_request = CollectRequest::new(
            Funds::new(1, 1, 1, 1, 1, 1),
            Funds::new(1, 0, 0, 0, 0, 0),
            vec![Piece::Blue, Piece::Brown, Piece::White],
            vec![Piece::Red, Piece::Green],
        );
        let response = Funds::collect(collect_request).unwrap_err();
        assert_eq!(response, CollectError::NotEnoughPiecesToDiscard);
    }

    #[test]
    fn cannot_discard_more_than_three() {
        let collect_request = CollectRequest::new(
            Funds::new(1, 1, 1, 1, 1, 1),
            Funds::new(1, 1, 1, 1, 1, 1),
            vec![Piece::Blue, Piece::Brown, Piece::White],
            vec![Piece::Red, Piece::Green, Piece::White, Piece::Brown],
        );
        let response = Funds::collect(collect_request).unwrap_err();
        assert_eq!(response, CollectError::CannotDiscardMoreThanThree);
    }

    #[test]
    fn correctly_convert_from_funds_to_vec_of_pieces() {
        let funds = Funds::new(1, 2, 3, 4, 5, 6);
        let pieces = Vec::<Piece>::from(funds.clone());

        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::Red).count(), 1);
        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::Green).count(), 2);
        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::Blue).count(), 3);
        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::Brown).count(), 4);
        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::White).count(), 5);
        assert_eq!(pieces.iter().clone().filter(|p| *p == &Piece::Golden).count(), 6);

        let new_funds = Funds::new_from_list(pieces);
        assert_eq!(funds, new_funds);
    }
}
