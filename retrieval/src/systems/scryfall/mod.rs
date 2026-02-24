use std::collections::HashMap;

use eyre::OptionExt;
use models::{
    Card, CardColour, CardID, CardIdentifiers, CollectorNumber, SetCode, filters::CardSearchFilters,
};
use serde_json::Value;

use crate::{NamedRetrievalSystem, RetrievalSystemTrait};

#[derive(Debug, Clone)]
pub struct ScryfallRetrievalSystem {}

impl NamedRetrievalSystem for ScryfallRetrievalSystem {}

impl ScryfallRetrievalSystem {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {})
    }
}

impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    #[allow(unused_variables)]
    async fn search_cards(
        &self,
        filters: CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<Card>> {
        let mut query = vec![];

        if let Some(name) = &filters.name {
            query.push(format!("name:{}", name));
        }

        if let Some(set_code) = &filters.set_code {
            query.push(format!("set:{}", set_code));
        }

        if let Some(color_identities) = &filters.color_identities {
            for color in color_identities {
                query.push(format!("c:{}", color));
            }
        }

        if let Some(text) = &filters.text {
            query.push(format!("t:{}", text));
        }

        if let Some(types) = &filters.types {
            for t in types {
                query.push(format!("type:{}", t));
            }
        }

        if let Some(subtypes) = &filters.subtypes {
            for s in subtypes {
                query.push(format!("type:{}", s));
            }
        }

        if let Some(supertypes) = &filters.supertypes {
            query.push(format!("type:{}", supertypes));
        }

        let query_string = query.join(" ");

        let page = skip.map(|s| s / 100).unwrap_or(1);
        let unique = "cards";
        let order = "name";
        let dir = "asc";
        let include_extras = false;

        let url = format!(
            "https://api.scryfall.com/cards/search?q={}&page={}&unique={}&order={}&dir={}&include_extras={}",
            query_string, page, unique, order, dir, include_extras
        );

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("gathers_cli/1.0"),
        );
        headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
        let client = reqwest::Client::new();
        let response = client.get(url).headers(headers).send().await?;
        let json: Value = response.json().await?;

        if let Some(error) = json.get("object").and_then(Value::as_str) {
            if error == "error" {
                let error_msg = json
                    .get("details")
                    .and_then(Value::as_str)
                    .unwrap_or("Unknown error");
                return Err(eyre::eyre!("Scryfall API error: {}", error_msg));
            }
        }

        let cards_array = json
            .get("data")
            .and_then(Value::as_array)
            .ok_or_eyre("Could not retrieve cards array")?;

        let limit = limit.unwrap_or(cards_array.len());
        let cards = cards_array
            .iter()
            .take(limit)
            .filter_map(|card| {
                let card = card.as_object()?;
                let card_name = card.get("name")?.as_str()?;
                let card_id = card.get("id")?.as_str()?;
                let set_code = card.get("set")?.as_str()?;
                let artist = card.get("artist")?.as_str()?;
                let rarity = card.get("rarity")?.as_str()?;
                let oracle_text = card.get("oracle_text")?.as_str()?;
                let collector_number = card.get("collector_number")?.as_str()?;

                let color_identity = card
                    .get("color_identity")
                    .and_then(Value::as_array)?
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|c| match *c {
                        "B" => CardColour::Black,
                        "U" => CardColour::Blue,
                        "W" => CardColour::White,
                        "G" => CardColour::Green,
                        "R" => CardColour::Red,
                        "C" => CardColour::Colourless,
                        _ => CardColour::Colourless,
                    })
                    .collect::<Vec<CardColour>>();

                let type_line = card.get("type_line")?.as_str()?;
                let mut types = vec![];
                let mut subtypes = vec![];
                let mut supertypes = vec![];

                let parts: Vec<&str> = type_line.split("—").map(|p| p.trim()).collect();
                if !parts.is_empty() {
                    let type_part = parts[0];
                    let type_tokens: Vec<&str> = type_part.split(' ').collect();
                    for token in type_tokens {
                        match token {
                            // TODO: add the rest
                            "Legendary" | "Basic" | "World" => supertypes.push(token.to_string()),
                            _ => types.push(token.to_string()),
                        }
                    }
                }
                if parts.len() > 1 {
                    let subtype_part = parts[1];
                    let subtype_tokens: Vec<&str> = subtype_part.split(' ').collect();
                    subtypes = subtype_tokens.iter().map(|s| s.to_string()).collect();
                }

                Some(Card::Magic(models::MagicCard {
                    name: card_name.to_string(),
                    set_code: set_code.to_string(),
                    artist: artist.to_string(),
                    color_identity,
                    id: card_id.to_string(),
                    rarity: rarity.to_string().into(),
                    text: oracle_text.to_string(),
                    card_identifiers: CardIdentifiers {
                        scryfall_id: card_id.to_string(),
                        id: card_id.to_string(),
                    },
                    collector_number: collector_number.to_string(),
                    subtypes,
                    supertypes,
                    types,
                }))
            })
            .collect::<Vec<Card>>();

        Ok(cards)
    }

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::Card>> {
        let mut result = HashMap::new();

        for id in ids {
            let url = format!("https://api.scryfall.com/cards/{}", id);
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("gathers_cli/1.0"),
            );
            headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
            let client = reqwest::Client::new();
            let response = client.get(url).headers(headers).send().await?;
            let json: Value = response.json().await?;

            let card_name = json
                .get("name")
                .and_then(Value::as_str)
                .ok_or_eyre("Could not retrieve name")?;
            let card_id = json
                .get("id")
                .and_then(Value::as_str)
                .ok_or_eyre("Could not retrieve id")?
                .to_string();

            let card = models::MagicCard {
                name: card_name.to_string(),
                set_code: json
                    .get("set")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve set")?
                    .to_string(),
                artist: json
                    .get("artist")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve artist")?
                    .to_string(),
                color_identity: json
                    .get("color_identity")
                    .and_then(Value::as_array)
                    .ok_or_eyre("Could not retrieve color identity")?
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<&str>>()
                    .iter()
                    .map(|c| match *c {
                        "B" => CardColour::Black,
                        "U" => CardColour::Blue,
                        "W" => CardColour::White,
                        "G" => CardColour::Green,
                        "R" => CardColour::Red,
                        "C" => CardColour::Colourless,
                        _ => CardColour::Colourless,
                    })
                    .collect::<Vec<CardColour>>(),
                id: card_id.clone(),
                rarity: json
                    .get("rarity")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve rarity")?
                    .to_string()
                    .into(),
                text: json
                    .get("oracle_text")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve oracle text")?
                    .to_string(),
                card_identifiers: CardIdentifiers {
                    scryfall_id: card_id.clone(),
                    id: card_id.clone(),
                },
                collector_number: json
                    .get("collector_number")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve collector number")?
                    .to_string(),
                subtypes: vec![],
                supertypes: vec![],
                types: vec![],
            };

            result.insert(id, models::Card::Magic(card));
        }

        Ok(result)
    }

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>> {
        // TODO: implement this
        Ok(vec![])
    }

    #[allow(unused_variables)]
    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>> {
        Ok(vec![])
    }

    async fn update_backend(&self) -> eyre::Result<bool> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_basic_card() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem {};
        let card = r
            .search_cards(
                CardSearchFilters {
                    name: Some("Panharmonicon".to_string()),
                    ..Default::default()
                },
                None,
                None,
            )
            .await?;

        assert_eq!(card.len(), 1);
        let card = card.first().expect("No card?");
        let card = if let Card::Magic(card) = card {
            card
        } else {
            panic!("Not a Magic card")
        };
        assert_eq!(card.name, "Panharmonicon");
        assert_eq!(card.color_identity, vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn get_cards_by_ids() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem {};
        let test_ids = vec![
            "998d0cc8-ca2a-41c3-ab65-d05c26ab8278".to_string(),
            "9a6cd6f6-ae6e-4a77-95ca-64c6882357d5".to_string(),
            "70dd138f-391a-4956-bc2a-fe186429c71a".to_string(),
        ];

        let result = r.get_cards_by_ids(test_ids).await?;

        assert_eq!(result.len(), 3);
        assert!(result.contains_key("998d0cc8-ca2a-41c3-ab65-d05c26ab8278"));
        assert!(result.contains_key("9a6cd6f6-ae6e-4a77-95ca-64c6882357d5"));
        assert!(result.contains_key("70dd138f-391a-4956-bc2a-fe186429c71a"));

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_types() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem::new()?;

        let creatures = r
            .search_cards(
                CardSearchFilters {
                    types: Some(vec!["Creature".to_string()]),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(!creatures.is_empty(), "Should find at least one creature");

        for card in creatures {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.types.is_empty(),
                "Card {} should have types",
                card.name
            );
        }

        let artifacts = r
            .search_cards(
                CardSearchFilters {
                    types: Some(vec!["Artifact".to_string()]),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(!artifacts.is_empty(), "Should find at least one artifact");

        for card in artifacts {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.types.is_empty(),
                "Card {} should have types",
                card.name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_subtypes() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem::new()?;

        let elves = r
            .search_cards(
                CardSearchFilters {
                    subtypes: Some(vec!["Elf".to_string()]),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(!elves.is_empty(), "Should find at least one elf");

        for card in elves {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.subtypes.is_empty(),
                "Card {} should have subtypes",
                card.name
            );
        }

        let wizards = r
            .search_cards(
                CardSearchFilters {
                    subtypes: Some(vec!["Wizard".to_string()]),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(!wizards.is_empty(), "Should find at least one wizard");

        for card in wizards {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.subtypes.is_empty(),
                "Card {} should have subtypes",
                card.name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_supertypes() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem::new()?;

        let legendary = r
            .search_cards(
                CardSearchFilters {
                    supertypes: Some("Legendary".to_string()),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(
            !legendary.is_empty(),
            "Should find at least one legendary card"
        );

        for card in legendary {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.supertypes.is_empty(),
                "Card {} should have supertypes",
                card.name
            );
        }

        let basic = r
            .search_cards(
                CardSearchFilters {
                    supertypes: Some("Basic".to_string()),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(!basic.is_empty(), "Should find at least one basic land");

        for card in basic {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.supertypes.is_empty(),
                "Card {} should have supertypes",
                card.name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_multiple_types() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem::new()?;

        let creatures_artifacts = r
            .search_cards(
                CardSearchFilters {
                    types: Some(vec!["Creature".to_string(), "Artifact".to_string()]),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(
            !creatures_artifacts.is_empty(),
            "Should find at least one creature or artifact"
        );

        for card in creatures_artifacts {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.types.is_empty(),
                "Card {} should have types",
                card.name
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_search_by_combined_filters() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem::new()?;

        let legendary_creatures = r
            .search_cards(
                CardSearchFilters {
                    types: Some(vec!["Creature".to_string()]),
                    subtypes: Some(vec!["Elf".to_string()]),
                    supertypes: Some("Legendary".to_string()),
                    ..Default::default()
                },
                None,
                Some(5),
            )
            .await?;

        assert!(
            !legendary_creatures.is_empty(),
            "Should find at least one legendary elf creature"
        );

        // Verify that all returned cards have the expected fields
        for card in legendary_creatures {
            let card = if let Card::Magic(card) = card {
                card
            } else {
                panic!("Not a Magic card")
            };

            assert!(
                !card.types.is_empty(),
                "Card {} should have types",
                card.name
            );
            assert!(
                !card.subtypes.is_empty(),
                "Card {} should have subtypes",
                card.name
            );
            assert!(
                !card.supertypes.is_empty(),
                "Card {} should have supertypes",
                card.name
            );
        }

        Ok(())
    }
}
