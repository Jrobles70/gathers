use criterion::{Criterion, criterion_group, criterion_main};
use models::{CardColour, Rarity, filters::CardSearchFilters};
use retrieval::{MagicSQLiteRetrievalSystem, RetrievalSystemTrait};
use std::hint::black_box;

fn bench_card_search_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let system = MagicSQLiteRetrievalSystem::new(None).unwrap();

    let mut group = c.benchmark_group("card_search");

    // Test 1: Simple name search
    group.bench_function("search_by_name", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                name: Some("Lightning Bolt".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 2: Search with color identity filter
    group.bench_function("search_by_color_identity", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                color_identities: Some(vec![CardColour::Red]),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 3: Search with artist filter
    group.bench_function("search_by_artist", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                artist: Some("Jason Chan".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 4: Search with text filter
    group.bench_function("search_by_text", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                text: Some("destroy target enchantment".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 5: Search with set code filter
    group.bench_function("search_by_set_code", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                set_code: Some("M20".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 6: Search with rarity filter
    group.bench_function("search_by_rarity", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                rarity: Some(Rarity::Rare),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 7: Search with multiple filters
    group.bench_function("search_with_multiple_filters", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                name: Some("Black Lotus".to_string()),
                color_identities: Some(vec![CardColour::Black]),
                rarity: Some(Rarity::Rare),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, None).await;
            let _ = black_box(result);
        })
    });

    // Test 8: Search with limit
    group.bench_function("search_with_limit", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                name: Some("Plains".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), None, Some(10)).await;
            let _ = black_box(result);
        })
    });

    // Test 9: Search with skip
    group.bench_function("search_with_skip", |b| {
        b.to_async(&rt).iter(|| async {
            let filters = CardSearchFilters {
                name: Some("Forest".to_string()),
                ..Default::default()
            };
            let result = system.search_cards(black_box(filters), Some(100), None).await;
            let _ = black_box(result);
        })
    });

    // Test 10: Get cards by IDs
    group.bench_function("get_cards_by_ids", |b| {
        b.to_async(&rt).iter(|| async {
            let ids = vec![
                "0003caab-9ff5-5d1a-bc06-976dd0457f19".to_string(),
                "0005d268-3fd0-5424-bc6b-573ecd713aa1".to_string(),
                "0006e8a4-4a4b-5e4e-8c4e-4e4e4e4e4e4e".to_string(),
            ];
            let result = system.get_cards_by_ids(black_box(ids)).await;
            let _ = black_box(result);
        })
    });

    // Test 11: Get sets
    group.bench_function("get_sets", |b| {
        b.to_async(&rt).iter(|| async {
            let result = system.get_sets().await;
            let _ = black_box(result);
        })
    });

    // Test 12: Bulk search cards
    group.bench_function("bulk_search_cards", |b| {
        b.to_async(&rt).iter(|| async {
            let cards = vec![
                ("M20".to_string(), "1".to_string()),
                ("M20".to_string(), "2".to_string()),
                ("M20".to_string(), "3".to_string()),
                ("M20".to_string(), "4".to_string()),
                ("M20".to_string(), "5".to_string()),
            ];
            let result = system.bulk_search_cards(black_box(cards)).await;
            let _ = black_box(result);
        })
    });

    group.finish();
}

criterion_group!(benches, bench_card_search_benchmark);
criterion_main!(benches);
