pub struct DummyRetrievalSystem {

}

use crate::RetrievalSystemTrait;

impl RetrievalSystemTrait for DummyRetrievalSystem {
    fn get_card(&self) -> models::Card {
        println!("Got request.");

        models::Card { name: "Test".to_string() }
    }
}
