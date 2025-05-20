use models::{filters::CardColour, Card};

#[derive(Debug, PartialEq)]
pub struct SqlCardIdentifiers {
    pub id: String,
    pub scryfall_id: String,
}

#[derive(Debug, PartialEq)]
pub struct SqlCard {
    pub id: String,
    pub name: String,
    pub set_code: String,
    pub rarity: String,
    pub artist: String,
    pub color_identity: String,
    pub text: String,
    pub card_identifiers: SqlCardIdentifiers,
}

impl From<SqlCard> for Card {
    fn from(value: SqlCard) -> Self {
        Card {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity.into(),
            artist: value.artist,
            color_identity: value
                .color_identity
                .chars()
                .filter_map(|c| match c {
                    'W' => Some(CardColour::White),
                    'U' => Some(CardColour::Blue),
                    'B' => Some(CardColour::Black),
                    'R' => Some(CardColour::Red),
                    'G' => Some(CardColour::Green),
                    ' ' => None,
                    ',' => None,
                    _ => Some(CardColour::Colourless),
                })
                .collect(),
            text: value.text,
            card_identifiers: models::CardIdentifiers {
                id: value.card_identifiers.id,
                scryfall_id: value.card_identifiers.scryfall_id,
            },
        }
    }
}
