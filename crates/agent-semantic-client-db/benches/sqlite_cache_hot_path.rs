use std::time::{SystemTime, UNIX_EPOCH};

use agent_semantic_client_db::ClientDb;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn sqlite_cache_hot_path(c: &mut Criterion) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let cache_root = std::env::temp_dir().join(format!("asp-client-db-bench-{unique}"));
    let db_path = ClientDb::default_path(&cache_root);
    let mut db = ClientDb::open_or_create(&db_path).expect("open benchmark db");
    let read_db = ClientDb::open_read_only_existing(&db_path)
        .expect("open read-only benchmark db")
        .expect("benchmark db exists");
    c.bench_function("sqlite_cache_hot_path/path_inspect", |b| {
        b.iter(|| {
            let report = ClientDb::inspect(black_box(&db_path));
            black_box(report.generation_count);
            black_box(&mut db);
        });
    });
    c.bench_function("sqlite_cache_hot_path/open_report", |b| {
        b.iter(|| {
            let report = read_db.inspect_open().expect("inspect open db");
            black_box(report.generation_count);
            black_box(&read_db);
        });
    });
    let _ = std::fs::remove_dir_all(cache_root);
}

criterion_group!(benches, sqlite_cache_hot_path);
criterion_main!(benches);
