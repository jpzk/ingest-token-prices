use chrono::NaiveDateTime;

use super::schema::{prices};

use diesel::sql_types::*;

#[derive(Queryable, Insertable, Debug)]
#[table_name="prices"]
pub struct Price { 
    pub dt: NaiveDateTime,
    pub base: String,
    pub in_usd: f32,
    pub in_eur: f32
}

#[derive(Queryable, Debug)]
pub struct Mapping { 
    pub symbol: String,
    pub name: String
}

#[derive(QueryableByName)]
pub struct Count { 
    #[sql_type = "Integer"]
    pub count: i32
}

