use amiko::serve;

#[test]
fn e2e_test() {
	use redis::*;

	std::thread::spawn(|| {
		serve();
	});
	std::thread::sleep(std::time::Duration::from_millis(100));

	let client = Client::open("redis://127.0.0.1:6379").unwrap();
	let mut conn = client.get_connection().unwrap();

	let result: RedisResult<Option<String>> = conn.get("key");
	assert!(result.is_ok());
	assert!(result.unwrap().is_none());

	let result: RedisResult<()> = conn.set("key", "value");
	assert!(result.is_ok());

	let result: RedisResult<Option<String>> = conn.get("key");
	assert!(result.is_ok());
	assert!(result.unwrap() == Some("value".to_string()));

	let result: RedisResult<()> = conn.set("key", "value2");
	assert!(result.is_ok());

	let result: RedisResult<Option<String>> = conn.get("key");
	assert!(result.is_ok());
	assert!(result.unwrap() == Some("value2".to_string()));

	let result: RedisResult<Vec<String>> = conn.keys("*");
	assert!(result.is_ok());
	assert!(result.unwrap() == vec!["key".to_string()]);
}
