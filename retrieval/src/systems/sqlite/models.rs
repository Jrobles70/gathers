use models::{Card, CardColour};

#[derive(Debug, PartialEq, Clone)]
pub struct SqlCardIdentifiers {
    pub id: String,
    pub scryfall_id: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SqlCard {
    pub id: String,
    pub name: String,
    pub set_code: String,
    pub rarity: String,
    pub artist: String,
    pub color_identity: String,
    pub text: String,
    pub card_identifiers: SqlCardIdentifiers,
    pub collector_number: String,
}

impl From<SqlCard> for Card {
    fn from(value: SqlCard) -> Self {
        let colours: Vec<CardColour> = value
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
                _ => None,
            })
            .collect();
        let colours = if colours.is_empty() {
            vec![CardColour::Colourless]
        } else {
            colours
        };
        Card {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity.into(),
            artist: value.artist,
            color_identity: colours,
            text: value.text,
            card_identifiers: models::CardIdentifiers {
                id: value.card_identifiers.id,
                scryfall_id: value.card_identifiers.scryfall_id,
            },
            collector_number: value.collector_number,
        }
    }
}
