table! {
    mapping (symbol) {
        symbol -> Text,
        name -> Text,
    }
}

table! {
    prices (dt, base) {
        dt -> Timestamp,
        base -> Text,
        in_usd -> Float,
        in_eur -> Float,
    }
}

allow_tables_to_appear_in_same_query!(
    mapping,
    prices,
);
