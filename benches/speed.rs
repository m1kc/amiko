use redis::*;

use amiko::serve;


fn main() {
	std::thread::spawn(|| {
		serve();
	});
	std::thread::sleep(std::time::Duration::from_millis(100));

	let client = Client::open("redis://127.0.0.1:6379").unwrap();
	let mut conn = client.get_connection().unwrap();

	/* Test 1: 20000 writes of different keys */

	let count = 20000;
	println!("Test 1: {} writes of different keys", count);

	use std::time::Instant;
    let now = Instant::now();

	{
		for i in 0..count {
			let key = format!("key{}", i);
			let value = format!("value{}", i);
			let result: RedisResult<()> = conn.set(&key, &value);
			assert!(result.is_ok());
		}
	}

	let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}


// #[macro_use]
// extern crate criterion;
// use criterion::Criterion;
// fn bench_method1(c: &mut Criterion) {
// 	c.bench_function("fib 20", |b| b.iter(|| 2 + 4));
// }
//
// fn bench_method2(c: &mut Criterion) {
// }
//
// criterion_group!(benches, bench_method1, bench_method2);
// criterion_main!(benches);
