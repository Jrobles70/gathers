use eyre::{Result, WrapErr};
use scraper::{Html, Selector};

use crate::systems::pokemon::scraper::common::{format_exp_number, format_id};
use crate::systems::pokemon::scraper::models::{Card, Pokemon, SerebiiSet};

const BASE: &str = "https://www.serebii.net";

pub async fn scrape_normal_sets(
    client: &reqwest::Client,
    count: Option<usize>,
) -> Result<Vec<SerebiiSet>> {
    let url = format!("{BASE}/card/english.shtml");
    scrape_set_list(client, &url, count, false).await
}

pub async fn scrape_promo_sets(
    client: &reqwest::Client,
    count: Option<usize>,
) -> Result<Vec<SerebiiSet>> {
    let url = format!("{BASE}/card/engpromo.shtml");
    scrape_set_list(client, &url, count, true).await
}

pub async fn scrape_set_details(
    client: &reqwest::Client,
    set_url: &str,
    exp_name: &str,
) -> Result<(String, Vec<Card>)> {
    let html = client
        .get(set_url)
        .send()
        .await
        .wrap_err_with(|| format!("fetching set page {set_url}"))?
        .text()
        .await?;
    let document = Html::parse_document(&html);
    let logo = extract_logo(&document);
    let cards = extract_cards(&document, exp_name);
    Ok((logo, cards))
}

pub async fn scrape_pokedex(client: &reqwest::Client) -> Result<Vec<Pokemon>> {
    let url = format!("{BASE}/pokemon/nationalpokedex.shtml");
    let html = client
        .get(&url)
        .send()
        .await
        .wrap_err("fetching Serebii Pokedex")?
        .text()
        .await?;
    let document = Html::parse_document(&html);

    let table_sel = Selector::parse(".dextable").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();

    let table = match document.select(&table_sel).next() {
        Some(t) => t,
        None => return Ok(vec![]),
    };

    let mut pokemon = Vec::new();
    for row in table.select(&tr_sel).skip(2) {
        let cells: Vec<_> = row.select(&td_sel).collect();
        if cells.len() < 3 {
            continue;
        }
        let num_str = cells[0].text().collect::<String>().replace('#', "");
        let id = match num_str.trim().parse::<i64>() {
            Ok(n) => n,
            Err(_) => continue,
        };
        let name = cells[2].text().collect::<String>().trim().to_string();
        if !name.is_empty() {
            pokemon.push(Pokemon { id, name });
        }
    }
    Ok(pokemon)
}

async fn scrape_set_list(
    client: &reqwest::Client,
    url: &str,
    count: Option<usize>,
    is_promo: bool,
) -> Result<Vec<SerebiiSet>> {
    let html = client
        .get(url)
        .send()
        .await
        .wrap_err_with(|| format!("fetching Serebii set list {url}"))?
        .text()
        .await?;

    // Extract all data synchronously into owned Strings so all !Send scraper/tendril
    // types are dropped before the first .await below.
    let raw: Vec<(String, String, String, i64)> = {
        let document = Html::parse_document(&html);

        let table_sel = Selector::parse("table").unwrap();
        let tr_sel = Selector::parse("tr").unwrap();
        let td_sel = Selector::parse("td").unwrap();
        let a_sel = Selector::parse("a").unwrap();
        let img_sel = Selector::parse("img").unwrap();

        let table = match document.select(&table_sel).next() {
            Some(t) => t,
            None => return Ok(vec![]),
        };
        let all_rows: Vec<_> = table.select(&tr_sel).collect();
        let data_rows = all_rows.iter().skip(1);
        let data_rows: Box<dyn Iterator<Item = _>> = match count {
            Some(n) => Box::new(data_rows.take(n)),
            None => Box::new(data_rows),
        };

        let mut out = Vec::new();
        for row in data_rows {
            let cells: Vec<_> = row.select(&td_sel).collect();

            let entry: Option<(String, String, String, i64)> = if is_promo {
                if cells.len() < 3 {
                    continue;
                }
                let anchor = cells[0].select(&a_sel).next();
                let sym_img = cells[2].select(&img_sel).next();
                match (anchor, sym_img) {
                    (Some(a), Some(i)) => {
                        let href = a.value().attr("href").unwrap_or("");
                        let name = a.text().collect::<String>().trim().to_string();
                        let page = make_abs(href);
                        let sym = make_abs(i.value().attr("src").unwrap_or(""));
                        let count_str = cells[1].text().collect::<String>();
                        let n = count_str.trim().parse::<i64>().unwrap_or(0);
                        Some((name, page, sym, n))
                    }
                    _ => None,
                }
            } else {
                if cells.len() < 4 {
                    continue;
                }
                let anchor = cells[2].select(&a_sel).next();
                let sym_img = cells[0].select(&img_sel).next();
                match (anchor, sym_img) {
                    (Some(a), Some(i)) => {
                        let href = a.value().attr("href").unwrap_or("");
                        let name = a.text().collect::<String>().trim().to_string();
                        let page = make_abs(href);
                        let sym = make_abs(i.value().attr("src").unwrap_or(""));
                        let count_str = cells[3].text().collect::<String>();
                        let n = count_str.trim().parse::<i64>().unwrap_or(0);
                        Some((name, page, sym, n))
                    }
                    _ => None,
                }
            };

            if let Some(e) = entry {
                if !e.0.is_empty() {
                    out.push(e);
                }
            }
        }
        out
        // document, table, all_rows, data_rows all dropped here
    };

    // Only Send types remain — safe to .await
    let mut sets = Vec::new();
    for (name, page_url, symbol_url, num_cards) in raw {
        let logo = fetch_logo(client, &page_url).await.unwrap_or_default();
        sets.push(SerebiiSet {
            name,
            page: page_url,
            logo,
            symbol: symbol_url,
            number_of_cards: num_cards,
        });
    }

    Ok(sets)
}

async fn fetch_logo(client: &reqwest::Client, set_url: &str) -> Result<String> {
    let html = client.get(set_url).send().await?.text().await?;
    let document = Html::parse_document(&html);
    Ok(extract_logo(&document))
}

fn extract_logo(document: &Html) -> String {
    let sel = Selector::parse("main img").unwrap();
    document
        .select(&sel)
        .next()
        .and_then(|img| img.value().attr("src"))
        .map(make_abs)
        .unwrap_or_default()
}

fn extract_cards(document: &Html, exp_name: &str) -> Vec<Card> {
    let table_sel = Selector::parse(".dextable").unwrap();
    let tr_sel = Selector::parse("tr").unwrap();
    let td_sel = Selector::parse("td").unwrap();
    let a_sel = Selector::parse("a").unwrap();
    let img_sel = Selector::parse("img").unwrap();

    let table = match document.select(&table_sel).next() {
        Some(t) => t,
        None => return vec![],
    };

    let mut cards = Vec::new();

    for row in table.select(&tr_sel).skip(1) {
        let cells: Vec<_> = row.select(&td_sel).collect();
        if cells.len() != 4 {
            continue;
        }

        let mut raw_num = cells[0].text().collect::<String>();
        if let Some(a) = cells[0].select(&a_sel).next() {
            let set_text = a.text().collect::<String>();
            raw_num = raw_num.replace(&set_text, "");
        }
        let card_num = format_exp_number(raw_num.trim());

        let rarity = cells[0]
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(parse_rarity)
            .unwrap_or_else(|| "Common".to_string());

        let img_url = cells[1]
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(|src| make_abs(src).replace("/th", ""))
            .unwrap_or_default();

        let name = cells[2].text().collect::<String>().trim().to_string();
        if name.is_empty() {
            continue;
        }

        let energy = cells[3]
            .select(&img_sel)
            .next()
            .and_then(|img| img.value().attr("src"))
            .map(parse_energy)
            .unwrap_or_default();

        let mut card_id = format_id(exp_name, &name, &card_num);
        card_id = card_id.replace(' ', "-");

        cards.push(Card {
            card_id,
            id_tcgp: 0,
            name,
            exp_code_tcgp: String::new(),
            exp_id_tcgp: String::new(),
            exp_name: exp_name.to_string(),
            exp_card_number: card_num,
            rarity,
            img: img_url,
            energy_type: Some(energy),
            ..Default::default()
        });
    }

    cards
}

fn make_abs(src: &str) -> String {
    if src.starts_with("http") {
        src.to_string()
    } else {
        format!("{BASE}{src}")
    }
}

fn parse_rarity(src: &str) -> String {
    if src.contains("common") {
        "Common"
    } else if src.contains("uncommon") {
        "Uncommon"
    } else if src.contains("holographic") {
        "Holo Rare"
    } else if src.contains("lvxrar") {
        "Ultra Rare"
    } else {
        "Common"
    }
    .to_string()
}

fn parse_energy(src: &str) -> String {
    if src.contains("grass") {
        "Grass"
    } else if src.contains("fire") {
        "Fire"
    } else if src.contains("water") {
        "Water"
    } else if src.contains("electric") {
        "Lightning"
    } else if src.contains("psychic") {
        "Psychic"
    } else if src.contains("fighting") {
        "Fighting"
    } else if src.contains("darkness") {
        "Darkness"
    } else if src.contains("metal") {
        "Metal"
    } else if src.contains("dragon") {
        "Dragon"
    } else if src.contains("colorless") {
        "Colorless"
    } else {
        ""
    }
    .to_string()
}
