use clap::{Parser, ValueEnum};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use models::{CardColour, Rarity};
use retrieval::{RetrievalSystem, RetrievalSystemTrait};
use sha2::Digest;
use std::io::Write;
use std::path::PathBuf;

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
    subtype: Option<Vec<String>>,

    #[clap(long)]
    supertype: Option<String>,

    #[clap(long)]
    types: Option<Vec<String>>,

    #[clap(short, long)]
    download: bool,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let retrieval_db_path = std::env::var("RETRIEVAL_DB_PATH")
        .ok()
        .or_else(|| Some("AllPrintings.db".to_string()));

    if args.download {
        download_database(retrieval_db_path.clone()).await?;
        return Ok(());
    }

    let db_path = PathBuf::from(retrieval_db_path.as_deref().unwrap_or("AllPrintings.db"));
    if !db_path.exists() {
        println!("Database file not found at: {:?}", db_path);
        println!("Would you like to download it? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input.trim().to_lowercase() == "y" {
            download_database(retrieval_db_path.clone()).await?;
        } else {
            println!("Please download the database first or use the --download flag.");
            return Ok(());
        }
    }

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
        Some(args.color.iter().map(|c| c.into()).collect())
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
                subtypes: args.subtype,
                supertypes: args.supertype,
                types: args.types,
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
            card.supertypes.join(","),
            card.types.join(","),
            card.subtypes.join(","),
        );
    }

    Ok(())
}

async fn download_database(db_path: Option<String>) -> eyre::Result<()> {
    let download_url = "https://mtgjson.com/api/v5/AllPrintings.sqlite";
    let crc_url = "https://mtgjson.com/api/v5/AllPrintings.sqlite.sha256";

    let persistent_path = if let Some(ref path) = db_path
        && path.starts_with('.')
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let persistent_dir = PathBuf::from(home).join(".local/share/gathers");
        std::fs::create_dir_all(&persistent_dir)?;
        persistent_dir.join("AllPrintings.db")
    } else {
        PathBuf::from(db_path.as_deref().unwrap_or("AllPrintings.db"))
    };

    println!("Retrieving sha256 file to check if download is needed.");

    let crc_response = reqwest::get(crc_url).await?;
    let crc_content = crc_response.text().await?;
    let remote_crc = crc_content.trim().to_lowercase();

    let local_crc = if persistent_path.exists() {
        let data = std::fs::read(&persistent_path)?;
        let mut hasher = sha2::Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        Some(hex::encode(result))
    } else {
        None
    };

    if local_crc.as_ref() == Some(&remote_crc) {
        println!("Database is already up to date (CRC: {}).", remote_crc);
        return Ok(());
    }

    println!("Downloading AllPrintings.db to: {:?}", persistent_path);

    let client = reqwest::Client::new();
    let response = client.get(download_url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {percent}% ({eta_precise}) {bytes} / {total_bytes}")
        ?.progress_chars("#>-"));

    let mut file = std::fs::File::create(&persistent_path)?;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk: Vec<u8> = chunk?.to_vec();
        file.write_all(&chunk)?;
        pb.inc(chunk.len() as u64);
    }

    pb.finish_with_message("Download complete");

    let data = std::fs::read(&persistent_path)?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    let downloaded_crc = hex::encode(result);

    if downloaded_crc == remote_crc {
        println!(
            "Database downloaded successfully (CRC: {}).",
            downloaded_crc
        );
    } else {
        println!(
            "Warning: CRC mismatch. Downloaded CRC: {}, Expected CRC: {}",
            downloaded_crc, remote_crc
        );
    }

    Ok(())
}
