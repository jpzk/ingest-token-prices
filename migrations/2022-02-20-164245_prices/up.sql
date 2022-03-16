-- Your SQL goes here

CREATE TABLE prices (
  dt DATETIME NOT NULL,
  base VARCHAR NOT NULL,
  in_usd FLOAT NOT NULL,
  in_eur FLOAT NOT NULL,
  PRIMARY KEY(dt, base)
)
