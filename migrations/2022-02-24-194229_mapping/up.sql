CREATE TABLE mapping (
  symbol VARCHAR NOT NULL PRIMARY KEY,
  name VARCHAR NOT NULL
);

INSERT INTO mapping VALUES ("BTC", "bitcoin");
INSERT INTO mapping VALUES ("ETH", "ethereum");
INSERT INTO mapping VALUES ("XMR", "monero");

