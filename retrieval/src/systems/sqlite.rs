pub struct SQLiteRetrievalSystem {}

use rusqlite::Connection;
use models::filters::CardSearchFilters;

use crate::RetrievalSystemTrait;

impl RetrievalSystemTrait for SQLiteRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<models::Card> {
        let conn = Connection::open("/home/mihail/AllPrintings.sqlite")?;
        let mut stmt = conn.prepare(format!("SELECT name FROM cards WHERE name LIKE '%{}%' LIMIT 1", filters.card_name.unwrap_or("".to_string())).as_str())?;
        let mut user_iter = stmt.query_map([], |row| {
            Ok(models::Card{
                name: row.get(0)?,
            })
        })?;

        match user_iter.next() {
            Some(u) => Ok(u?),
            None => Ok(models::Card { name: "ABC".to_string() })
        }
    }
}
