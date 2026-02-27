use eyre::{Result, eyre};
use regex::Regex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Adapted from https://github.com/vikkumar2021/RiftboundCardDatabase

const BASE_URL: &str = "https://riftbound.leagueoflegends.com";
const GALLERY_PATH: &str = "/en-us/card-gallery/";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SimplifiedCard {
    pub id: Option<String>,
    pub name: Option<String>,
    pub code: Option<String>,
    pub collector_number: Option<Value>,
    pub set: Option<String>,
    pub set_name: Option<String>,
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    pub type_id: Option<String>,
    pub rarity: Option<String>,
    pub rarity_id: Option<String>,
    pub image_url: Option<String>,
    pub orientation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub might: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ability_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artists: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub super_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_hash: Option<String>,
}

pub struct RiftboundCardFetcher {
    client: Client,
    pub build_id: Option<String>,
}

impl RiftboundCardFetcher {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()?;
        Ok(Self {
            client,
            build_id: None,
        })
    }

    pub fn fetch_build_id(&mut self) -> Result<String> {
        let url = format!("{}{}", BASE_URL, GALLERY_PATH);
        let html = self.client.get(&url).send()?.error_for_status()?.text()?;
        let re = Regex::new(r"/_next/static/([^/]+)/_buildManifest\.js")?;
        match re.captures(&html) {
            Some(cap) => {
                let build_id = cap[1].to_string();
                self.build_id = Some(build_id.clone());
                Ok(build_id)
            }
            None => Err(eyre!("Could not find build ID in HTML")),
        }
    }

    pub fn fetch_card_data(&mut self) -> Result<Value> {
        if self.build_id.is_none() {
            self.fetch_build_id()?;
        }
        let build_id = self.build_id.as_ref().unwrap();
        let url = format!("{BASE_URL}/_next/data/{build_id}/en-us/card-gallery.json");
        let data: Value = self.client.get(&url).send()?.error_for_status()?.json()?;
        Ok(data)
    }

    pub fn extract_cards(data: &Value) -> Result<Vec<Value>> {
        let items = data["pageProps"]["page"]["blades"][2]["cards"]["items"]
            .as_array()
            .ok_or_else(|| eyre!("Could not extract cards from data structure"))?;
        let cards = items.to_vec();
        Ok(cards)
    }

    pub fn simplify_card(card: &Value) -> SimplifiedCard {
        let mut s = SimplifiedCard {
            id: card["id"].as_str().map(str::to_string),
            name: card["name"].as_str().map(str::to_string),
            code: card["publicCode"].as_str().map(str::to_string),
            collector_number: {
                let v = &card["collectorNumber"];
                if v.is_null() { None } else { Some(v.clone()) }
            },
            set: card["set"]["value"]["id"].as_str().map(str::to_string),
            set_name: card["set"]["value"]["label"].as_str().map(str::to_string),
            card_type: card["cardType"]["type"][0]["label"]
                .as_str()
                .map(str::to_string),
            type_id: card["cardType"]["type"][0]["id"]
                .as_str()
                .map(str::to_string),
            rarity: card["rarity"]["value"]["label"]
                .as_str()
                .map(str::to_string),
            rarity_id: card["rarity"]["value"]["id"].as_str().map(str::to_string),
            image_url: card["cardImage"]["url"].as_str().map(str::to_string),
            orientation: card["orientation"].as_str().map(str::to_string),
            ..Default::default()
        };

        if let Some(values) = card["domain"]["values"].as_array() {
            s.domains = Some(
                values
                    .iter()
                    .filter_map(|d| d["label"].as_str().map(str::to_string))
                    .collect(),
            );
            s.domain_ids = Some(
                values
                    .iter()
                    .filter_map(|d| d["id"].as_str().map(str::to_string))
                    .collect(),
            );
        }

        if card["energy"].is_object() {
            s.energy = json_id_to_string(&card["energy"]["value"]["id"]);
        }
        if card["might"].is_object() {
            s.might = json_id_to_string(&card["might"]["value"]["id"]);
        }
        if card["power"].is_object() {
            s.power = json_id_to_string(&card["power"]["value"]["id"]);
        }

        if card["tags"].is_object() {
            s.tags = Some(card["tags"]["tags"].clone());
        }

        if card["text"].is_object() {
            s.ability_html = card["text"]["richText"]["body"]
                .as_str()
                .map(str::to_string);
        }

        if let Some(values) = card["illustrator"]["values"].as_array() {
            s.artists = Some(
                values
                    .iter()
                    .filter_map(|a| a["label"].as_str().map(str::to_string))
                    .collect(),
            );
        }

        if let Some(super_types) = card["cardType"]["superType"].as_array() {
            s.super_types = Some(
                super_types
                    .iter()
                    .filter_map(|st| st["label"].as_str().map(str::to_string))
                    .collect(),
            );
        }

        if let Some(ref image_url) = s.image_url {
            for part in image_url.split('/') {
                if !(part.contains('-') && part.contains('x') && part.ends_with(".png")) {
                    continue;
                }
                if let Some(hash) = part.split('-').next() {
                    s.image_hash = Some(hash.to_string());
                    break;
                }
            }
        }

        s
    }

    pub fn fetch_all_simplified(&mut self) -> Result<Vec<SimplifiedCard>> {
        let data = self.fetch_card_data()?;
        let cards = Self::extract_cards(&data)?;
        let simplified: Vec<SimplifiedCard> = cards.iter().map(Self::simplify_card).collect();
        Ok(simplified)
    }
}

fn json_id_to_string(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}
