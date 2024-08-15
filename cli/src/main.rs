use core::fmt;

use retrieval::{RetrievalSystem, RetrievalSystemTrait};
use clap::{ValueEnum, Parser};

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Systems {
    Scryfall,
    Dummy,
    Sql,
}

impl fmt::Display for Systems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long, default_value= "scryfall")]
    system: Systems,

    #[clap(short, long)]
    card_name: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let retrieval = match args.system {
        Systems::Scryfall => RetrievalSystem::Scryfall(retrieval::ScryfallRetrievalSystem {}),
        Systems::Sql => RetrievalSystem::Database(retrieval::SQLiteRetrievalSystem {  }),
        _ => RetrievalSystem::Dummy(retrieval::DummyRetrievalSystem {  }),
    };
    println!(
        "Hello, world! {:?}",
        retrieval.get_card(models::filters::CardSearchFilters {
            card_name: Some(args.card_name),
            card_colours: Some(vec![models::filters::CardColour::White])
        }).await?
    );
    Ok(())
}
