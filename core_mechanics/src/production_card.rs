use super::bank;
use super::board::BuyOperationFail;
use super::piece::Piece;
use super::player;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionCard {
    pub cost: bank::Funds,
    pub produces: Piece,
    pub victory_points: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardId {
    id: u8,
}

impl CardId {
    pub fn new(id: u8) -> Self {
        Self { id }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identifiable<T, IdType = u8> {
    pub uid: IdType,
    pub data: T,
}

impl Identifiable<ProductionCard, CardId> {
    pub fn new(card: ProductionCard, uid: CardId) -> Self {
        Self { uid, data: card }
    }
}

impl ProductionCard {
    pub fn new(cost: bank::Funds, produces: Piece, victory_points: Option<u8>) -> Self {
        Self {
            cost,
            produces,
            victory_points,
        }
    }

    pub fn buy(
        player: player::Player,
        prod_card: ProductionCard,
    ) -> Result<bank::Funds, BuyOperationFail> {
        let mut funds_remaining = player.funds.clone();
        let mut new_missing_funds = bank::Funds::new(0, 0, 0, 0, 0, 0);
        let mut is_missing_funds = false;

        let production_funds =
            player::Player::get_funds_from_production_cards(player.production_cards);

        for (piece, cost) in prod_card.cost.funds {
            let produces = production_funds.funds.get(&piece).unwrap_or(&0);

            if *produces >= cost {
                continue;
            }

            let current_amount = match funds_remaining.funds.get(&piece) {
                None => 0,
                Some(i) => *i,
            };
            if current_amount + produces >= cost {
                funds_remaining
                    .funds
                    .insert(piece, current_amount - (cost - produces));
            } else {
                let goldens = *funds_remaining.funds.get(&Piece::Golden).unwrap();
                if current_amount + produces + goldens >= cost {
                    funds_remaining.funds.insert(piece, 0);
                    let r = goldens - (cost - current_amount - produces);
                    funds_remaining.funds.insert(Piece::Golden, r);
                } else {
                    is_missing_funds = true;
                    new_missing_funds
                        .funds
                        .insert(piece, cost - current_amount - produces);
                }
            }
        }

        if is_missing_funds {
            Err(BuyOperationFail::NotEnoughFunds(new_missing_funds))
        } else {
            Ok(funds_remaining)
        }
    }
}

#[cfg(test)]
mod tests {

    use player::PlayerId;

    use super::bank::Funds;

    use super::*;

    fn get_default_production() -> Vec<Identifiable<ProductionCard, CardId>> {
        vec![
            Identifiable::new(
                ProductionCard::new(get_default_cost(), Piece::Green, None),
                CardId::new(10),
            ),
            Identifiable::new(
                ProductionCard::new(get_default_cost(), Piece::Blue, None),
                CardId::new(20),
            ),
        ]
    }

    fn get_default_cost() -> bank::Funds {
        bank::Funds::new(0, 1, 2, 1, 1, 0)
    }

    #[test]
    fn can_buy_card() {
        let prod_card = ProductionCard::new(get_default_cost(), Piece::Red, None);

        let player_funds = bank::Funds::new(3, 2, 3, 1, 1, 1);
        let player = player::Player::new(PlayerId::new(1), player_funds, [].to_vec(), [].to_vec());

        let result = ProductionCard::buy(player, prod_card);

        let should_remain_funds = bank::Funds::new(3, 1, 1, 0, 0, 1);

        assert!(result.is_ok());

        assert_eq!(result.unwrap(), should_remain_funds);
    }

    #[test]
    fn cannot_buy_if_there_is_not_enough_funds() {
        let prod_card = ProductionCard::new(get_default_cost(), Piece::Red, None);

        let player_funds = bank::Funds::new(0, 0, 2, 1, 1, 0);
        let player = player::Player::new(PlayerId::new(1), player_funds, [].to_vec(), [].to_vec());

        let result = ProductionCard::buy(player, prod_card);

        assert!(result.is_err());

        assert_eq!(
            result.unwrap_err(),
            BuyOperationFail::NotEnoughFunds(Funds::new_from_list(vec!(Piece::Green)))
        );
    }

    #[test]
    fn can_buy_using_golden_piece_when_needed() {
        let prod_card = ProductionCard::new(get_default_cost(), Piece::Red, None);

        let player_funds = bank::Funds::new(0, 0, 1, 2, 1, 2);
        let player = player::Player::new(PlayerId::new(1), player_funds, [].to_vec(), [].to_vec());

        let result = ProductionCard::buy(player, prod_card);

        let should_remain_funds = bank::Funds::new(0, 0, 0, 1, 0, 0);

        assert_eq!(result.unwrap(), should_remain_funds);
    }

    #[test]
    fn prioritize_production_card_over_pieces() {
        let prod_card = ProductionCard::new(get_default_cost(), Piece::Red, None);

        let player_produces = get_default_production();
        let player_funds = bank::Funds::new(0, 1, 2, 2, 1, 0);
        let player =
            player::Player::new(PlayerId::new(1), player_funds, player_produces, [].to_vec());

        let result = ProductionCard::buy(player, prod_card);

        let should_remain_funds = bank::Funds::new(0, 1, 1, 1, 0, 0);

        assert_eq!(result.unwrap(), should_remain_funds);
    }

    #[test]
    fn use_production_card_with_pieces_and_golden_pieces() {
        let prod_card = ProductionCard::new(bank::Funds::new(0, 1, 2, 1, 1, 0), Piece::Red, None);

        let player_produces = get_default_production();
        let player_funds = bank::Funds::new(0, 1, 0, 2, 1, 1);
        let player = player::Player::new(
            PlayerId::new(1),
            player_funds,
            player_produces.clone(),
            [].to_vec(),
        );

        let result = ProductionCard::buy(player, prod_card);

        let should_remain_funds = bank::Funds::new(0, 1, 0, 1, 0, 0);

        assert_eq!(result.unwrap(), should_remain_funds);

        let prod_card = ProductionCard::new(bank::Funds::new(0, 2, 2, 1, 1, 0), Piece::Red, None);

        let player_funds = bank::Funds::new(0, 0, 0, 2, 1, 1);
        let player =
            player::Player::new(PlayerId::new(1), player_funds, player_produces, [].to_vec());

        let result = ProductionCard::buy(player, prod_card);

        assert!(result.is_err());
    }
}
