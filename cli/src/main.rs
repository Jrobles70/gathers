use clap::{Parser, ValueEnum};
use models::{CardColour, Rarity};
use retrieval::{RetrievalSystem, RetrievalSystemTrait};

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Systems {
    Scryfall,
    Sql,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long, default_value = "scryfall")]
    system: Systems,

    #[clap(short, long)]
    name: Option<String>,

    #[clap(short, long, default_value = "10")]
    limit: usize,

    #[clap(short, long, default_value = "0")]
    offset: usize,

    #[clap(short, long)]
    color: Vec<String>,

    #[clap(long)]
    set: Option<String>,

    #[clap(long)]
    collector_number: Option<String>,

    #[clap(short, long)]
    artist: Option<String>,

    #[clap(short, long)]
    text: Option<String>,

    #[clap(short, long)]
    rarity: Option<String>,

    #[clap(long)]
    subtype: Option<String>,

    #[clap(long)]
    supertype: Option<String>,

    #[clap(long)]
    types: Option<String>,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let retrieval_db_path = std::env::var("RETRIEVAL_DB_PATH")
        .ok()
        .or_else(|| Some("/home/mihail/.local/share/hometg/DB/AllPrintings.db".to_string()));

    let retrieval: RetrievalSystem = match args.system {
        Systems::Scryfall => {
            RetrievalSystem::ScryfallRetrievalSystem(retrieval::ScryfallRetrievalSystem {})
        }
        Systems::Sql => RetrievalSystem::MagicSQLiteRetrievalSystem(
            retrieval::MagicSQLiteRetrievalSystem::new(retrieval_db_path)?,
        ),
    };

    let color_identities: Option<Vec<CardColour>> = if args.color.is_empty() {
        None
    } else {
        Some(
            args.color
                .iter()
                .filter_map(|c| match c.to_lowercase().as_str() {
                    "w" | "white" => Some(CardColour::White),
                    "u" | "blue" => Some(CardColour::Blue),
                    "b" | "black" => Some(CardColour::Black),
                    "r" | "red" => Some(CardColour::Red),
                    "g" | "green" => Some(CardColour::Green),
                    "c" | "colourless" => Some(CardColour::Colourless),
                    "m" | "multicoloured" => Some(CardColour::Multicoloured),
                    _ => {
                        eprintln!("Warning: Unknown color '{}', ignoring", c);
                        None
                    }
                })
                .collect(),
        )
    };

    let rarity: Option<Rarity> = args.rarity.map(|r| r.into());

    let cards = retrieval
        .search_cards(
            models::filters::CardSearchFilters {
                name: args.name,
                color_identities,
                set_code: args.set,
                collector_number: args.collector_number,
                artist: args.artist,
                text: args.text,
                rarity,
                subtype: args.subtype,
                supertype: args.supertype,
                types: args.types,
                ..Default::default()
            },
            Some(args.offset),
            Some(args.limit),
        )
        .await?;

    if cards.is_empty() {
        println!("No cards found matching the criteria.");
        return Ok(());
    }

    println!("Found {} card(s):\n", cards.len());
    println!(
        "{:<30} {:<5} {:<10} {:<7} {:<25} {:<10} {:<15} {:<15} {:<15}",
        "Name", "Set", "Rarity", "Colors", "Artist", "Number", "Subtype", "Supertype", "Types"
    );
    println!("{}", "-".repeat(140));

    for card in cards {
        let color_str: String = card
            .color_identity
            .iter()
            .map(|c| format!("{}", c))
            .collect::<Vec<_>>()
            .join("");

        println!(
            "{:<30} {:<5} {:<10} {:<7} {:<25} {:<10} {:<15} {:<15} {:<15}",
            card.name,
            card.set_code,
            card.rarity.to_string().to_lowercase(),
            color_str,
            card.artist,
            card.collector_number,
            card.subtype,
            card.supertype,
            card.types
        );
    }

    Ok(())
}
