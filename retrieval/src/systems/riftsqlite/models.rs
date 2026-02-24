use models::riftbound::{CardDomain, RiftboundCard};

#[derive(Debug, PartialEq, Clone)]
pub struct SqlCard {
    pub id: String,
    pub name: String,
    pub set_code: String,
    pub rarity: String,
    pub artists: String,
    pub domains: String,
    pub text: String,
    pub image: String,
    pub collector_number: String,
}

impl SqlCard {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(SqlCard {
            id: row.get(0)?,
            name: row.get(1)?,
            set_code: row.get(2)?,
            domains: row.get(5)?,
            text: row.get(6)?,
            rarity: row.get(3)?,
            artists: row.get(4)?,
            image: row.get(7)?,
            collector_number: row.get(8)?,
        })
    }
}

impl From<SqlCard> for RiftboundCard {
    fn from(value: SqlCard) -> Self {
        let domains: Vec<CardDomain> = value
            .domains
            .split(",")
            .filter_map(|c| match c {
                "calm" | "Calm" => Some(CardDomain::Calm),
                "fury" | "Fury" => Some(CardDomain::Fury),
                "chaos" | "Chaos" => Some(CardDomain::Chaos),
                "order" | "Order" => Some(CardDomain::Order),
                "mind" | "Mind" => Some(CardDomain::Mind),
                "body" | "Body" => Some(CardDomain::Body),
                _ => None,
            })
            .collect();
        let domains = if domains.is_empty() {
            vec![CardDomain::Colorless]
        } else {
            domains
        };
        RiftboundCard {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity.into(),
            artists: value.artists,
            domains,
            text: value.text,
            image: value.image,
            collector_number: value.collector_number,
        }
    }
}
