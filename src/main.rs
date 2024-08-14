use models::Card;
use retrieval::{RetrievalSystem, RetrievalSystemTrait};

fn main() {
    let retrieval = RetrievalSystem::Dummy(retrieval::DummyRetrievalSystem{});
    println!("Hello, world! {:?}", retrieval.get_card());
}
