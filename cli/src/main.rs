use clap::{Parser, ValueEnum};
use retrieval::{RetrievalSystem, RetrievalSystemTrait};

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Systems {
    // Scryfall,
    // Dummy,
    Sql,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long, default_value = "scryfall")]
    system: Systems,

    #[clap(short, long)]
    card_name: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let retrieval = match args.system {
        // Systems::Scryfall => RetrievalSystem::Scryfall(retrieval::ScryfallRetrievalSystem {}),
        Systems::Sql => RetrievalSystem::Database(retrieval::SQLiteRetrievalSystem::new(None)?),
        // _ => RetrievalSystem::Dummy(retrieval::DummyRetrievalSystem {}),
    };
    println!(
        "{:?}",
        retrieval
            .search_cards(
                models::filters::CardSearchFilters {
                    name: Some(args.card_name),
                    // color_identities: Some(vec![models::filters::CardColour::White]),
                    ..Default::default()
                },
                Some(0),
                Some(2)
            )
            .await?
    );
    Ok(())
}
