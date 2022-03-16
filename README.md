# Simple Token Price Ingest

This is my first little `rust` project combining `Diesel ORM` with `SQLite` and `reqwest` to fetch and maintain token prices (last, and 30d historical) fetched from Coingecko. It will upsert duplicate prices.

## Setup

Add token mappings for the symbols you want to ingest via Coingecko in the migration in `migrations/2022-02-24-194229_mapping/up.sql`. Without modifications the file contains:

```sql
CREATE TABLE mapping (
  symbol VARCHAR NOT NULL PRIMARY KEY,
  name VARCHAR NOT NULL
);

INSERT INTO mapping VALUES ("BTC", "bitcoin");
INSERT INTO mapping VALUES ("ETH", "ethereum");
INSERT INTO mapping VALUES ("XMR", "monero");
```

After the migrations are customized you can create the SQLite database with `diesel migration run`. You sould end up with a fresh `prices.sqlite`.

## Ingest prices on-demand

You can either run to ingest the latest prices for the tokens set up above with `RUST_LOG=info cargo run last`, or ingest the last 30d of hourly prices `RUST_LOG=info cargo run historical`. It will upsert existing prices.

## Ingest prices with built-in scheduler

If you want to have a long-running process that schedules scraping prices internally then just run `RUST_LOG=info cargo run schedule`. It will scrape last prices once per hour, and it will parse scrape 30d historical prices once per day. It will upsert existing prices.
