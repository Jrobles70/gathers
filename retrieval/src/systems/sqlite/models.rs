use models::{CardColour, MagicCard};

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
    pub subtype: String,
    pub supertype: String,
    pub types: String,
}

impl SqlCard {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(SqlCard {
            id: row.get(0)?,
            name: row.get(1)?,
            set_code: row.get(2)?,
            color_identity: row.get(5)?,
            text: row.get(6)?,
            rarity: row.get(3)?,
            artist: row.get(4)?,
            card_identifiers: SqlCardIdentifiers {
                scryfall_id: row.get(7)?,
                id: row.get(0)?,
            },
            collector_number: row.get(8)?,
            subtype: row.get(9)?,
            supertype: row.get(10)?,
            types: row.get(11)?,
        })
    }
}

impl From<SqlCard> for MagicCard {
    fn from(value: SqlCard) -> Self {
        let colours: Vec<CardColour> = value
            .color_identity
            .chars()
            .filter_map(|c| match c {
                'W' | 'w' => Some(CardColour::White),
                'U' | 'u' => Some(CardColour::Blue),
                'B' | 'b' => Some(CardColour::Black),
                'R' | 'r' => Some(CardColour::Red),
                'G' | 'g' => Some(CardColour::Green),
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
        let subtypes = value
            .subtype
            .split(",")
            .map(|t| t.trim().to_string())
            .collect();
        let supertypes = value
            .supertype
            .split(",")
            .map(|t| t.trim().to_string())
            .collect();
        let types = value
            .types
            .split(",")
            .map(|t| t.trim().to_string())
            .collect();
        MagicCard {
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
            subtypes,
            supertypes,
            types,
        }
    }
}
