# amiko

An in-memory database, compatible with Redis protocol and clients. Written in Rust.

🔧 Stability: **early alpha**

⚠️ Hobby project, not for serious use. You have been warned.

_"Amiko" means "friend" in Esperanto. Stress falls on "i", like in Spanish "amigo"._

## Table of Contents

1. [Build](#build)
2. [Usage](#usage)
3. [Compatibility](#compatibility)
4. [Performance](#performance)
5. [License](#license)

## Build

```
cargo build --release
```

## Usage

Just run the `amiko` binary from the `target/` directory. Currently, it always binds to `127.0.0.1:6379`.

Default log level is `info`. Other available levels: `off`, `error`, `warn`, `info`, `debug`, `trace`. To change: `AMIKO_LOG=<level>`.

## Compatibility

Amiko is intended to be a drop-in replacement — your existing Redis clients, apps and libraries should work with Amiko without modifications.

The following commands are supported:

* `SET <key> <value>`
* `GET <key>`
* `DEL <key>`
* `KEYS <pattern>` (all Redis patterns should work)
* `FLUSHDB [SYNC|ASYNC]` (always synchronous, optional argument is parsed but ignored)
* `PING [msg]`, `ECHO <msg>`, `QUIT`

RESP3 is not supported at this time.

## Performance

Amiko is lightweight and has low memory footprint. Binary size is around 6 MB,
memory footprint is around ~2 MB (on an empty database).

Speed-wise, I expect it to be fairly speedy but not Redis speedy, at least in theory. In practice,
current version of Amiko was even a bit faster on some tests.

```
# Redis 7.2.4
Test 1: 20000 writes of different keys
Elapsed: 721.67ms
# amiko
Test 1: 20000 writes of different keys
Elapsed: 554.77ms
```

While there are no benchmarks, here are some general thoughts:

- Concurrent reads are lock-free and should be very fast.
- Writes involve mutex-like locking; this should be fast but not *that* fast.
- Amiko uses traditional threads, which means it may not play well with large number of clients. Tens of clients with heavy load should be fine, though.

## License

GNU GPL v3
