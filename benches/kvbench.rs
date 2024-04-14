use amiko::kv::Database;

#[macro_use]
extern crate criterion;
use criterion::Criterion;


fn bench_method1(c: &mut Criterion) {
	let mut db = Database::default();

	c.bench_function("key insert", |b| {
		b.iter(|| {
			let n = rand::random::<u32>();
			let key = format!("key{}", n).into_bytes();
			let value = format!("value{}", n).into_bytes();
			db.insert(key, value);
			let key = format!("key{}", n).into_bytes();
			let value = format!("value{}", n).into_bytes();
			assert_eq!(db.get(&key), Some(value));
		});
	});
}

criterion_group!(benches, bench_method1);
criterion_main!(benches);
