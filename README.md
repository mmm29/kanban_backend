### Run
Run with in-memory storage:
```bash
cargo run
```

To enable debug logging set `RUST_LOG=debug` environment variable:
```bash
RUST_LOG=debug cargo run
```

To enable PostgreSQL persistent storage set `DATABASE` environment variable:
```bash
DATABASE=postgres://user:password@host:port/name cargo run
```