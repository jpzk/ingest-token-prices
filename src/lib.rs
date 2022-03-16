#[macro_use]
extern crate diesel;
extern crate dotenv;

pub mod schema;
pub mod models;

// this module contains all operations logic (everything except fetching/parsing)
pub mod operations { 
    use job_scheduler::{JobScheduler, Job};
    use log::{trace, info, warn};

    use diesel::prelude::*;
    use diesel::sqlite::SqliteConnection;
    use diesel::Connection;

    use dotenv::dotenv;
    use std::env;
    use std::error::Error;
    use std::time::Duration;

    use crate::fetch::CoingeckoProvider;
    use crate::fetch::DatabaseSymbolToName;
    use crate::models::{Price, Count};
    use crate::schema::mapping::dsl::*;
    use crate::schema::prices::dsl::*;

    #[derive(Debug)]
    pub enum Mode {
        Last, 
        Historical
    }

    // operations supported; callable by clap 
    pub fn historical() {
        match ingest(Mode::Historical) {
            Ok(_) => info!("Historical insert completed"),
            Err(e) => warn!("{}", e) 
        }
    }
    pub fn last() { 
        match ingest(Mode::Last) {
            Ok(_) => info!("Current insert completed"),
            Err(e) => warn!("{}",e)
        }
    }
    pub fn delete() {
        let conn = establish_connection();
        match diesel::delete(prices).execute(&conn) {
            Ok(_) => (),
            Err(e) => warn!("{}",e)
        }
    }
    pub fn schedule() -> () { 
        info!("Scheduler started");
        let mut sched = JobScheduler::new();
        sched.add(Job::new("* * 0 * * * *".parse().unwrap(), || {historical()}));
        sched.add(Job::new("* 0 * * * * *".parse().unwrap(), || {last()}));
        loop {
            sched.tick();
            std::thread::sleep(Duration::from_millis(500));
        }
    }
    pub fn stats() -> () {
        let conn = establish_connection();
        let count: Count = diesel::sql_query("SELECT COUNT(*) as count FROM prices")
            .get_result(&conn).unwrap();
        info!("Saved {} prices", &count.count)
    }

    // internal functions
    fn establish_connection() -> SqliteConnection {
        trace!("Establishing connection to the database");
        dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }
    fn fetch_one(sym: String, provider: &CoingeckoProvider, mode: &Mode) 
        -> Result<Vec<Price>, Box<dyn Error>> {
        match mode {
            Mode::Last => { 
                info!("Fetch last for symbol {} and mode {:?}", &sym, &mode); 
                let price = provider.fetch_price(&sym);
                price.map(|p| { 
                    let mut v = Vec::new();
                    v.push(p);
                    v
                })
            }, 
            Mode::Historical => {
                info!("Fetch historical for symbol {} and mode {:?}", &sym, &mode);
                provider.fetch_historical_price(&sym)
            }
        }
    }
    fn ingest(mode: Mode) -> Result<(), Box<dyn Error>> { 
        trace!("Called ingest with mode {:?}", &mode);
        let conn = establish_connection();
        let resolver = DatabaseSymbolToName::new(&conn);
        let provider = CoingeckoProvider::new(&resolver);
        let symbols = mapping.select(symbol).load::<String>(&conn)?;
        let mut result: Vec<Price> = Vec::new();
        for sym in symbols {
            let ps = fetch_one(sym, &provider, &mode)?;
            for p in ps { 
                result.push(p)
            }
        }
        info!("Inserting {} prices", result.len());
        create_new_prices(&conn, result)
    }
    fn create_new_prices(conn: &SqliteConnection, ps: Vec<Price>) -> 
        Result<(), Box<dyn std::error::Error>> { 
        info!("Upserting {} prices", ps.len());
        diesel::replace_into(prices)
            .values(ps)
            .execute(conn)?;
        Ok(())
    }
}

pub mod fetch {
    use diesel::sqlite::SqliteConnection;
    use diesel::prelude::*;
    use chrono::NaiveDateTime;
    use reqwest::blocking::Client;
    use serde_json::{Value, from_value};
    use std::error::Error;

    use crate::models::{Price, Mapping};
    use crate::schema::mapping::dsl::*;

    pub struct DatabaseSymbolToName<'a> {
        conn: &'a SqliteConnection
    }
    impl<'a> DatabaseSymbolToName<'a> { 
        pub fn new(conn: &'a SqliteConnection) -> Self { 
            Self { conn }
        }
        fn resolve(&self, sym: &str) -> Result<String, Box<dyn Error>> {
            let m = mapping
                .filter(symbol.eq(sym))
                .limit(1)
                .load::<Mapping>(self.conn)?;
            let f = m.first().map(|v| v.name.to_owned());
            Ok(f.unwrap()) // dirty I know no need to add custom error trait
        }
    }
    pub struct CoingeckoProvider<'a> {
        client: Client,
        resolver: &'a DatabaseSymbolToName<'a>
    }
    impl<'a> CoingeckoProvider<'a> {
        pub fn new(resolver: &'a DatabaseSymbolToName) -> Self {
            Self { client: Client::new(), resolver}
        }
        fn retrieve(&self, url: &str) -> Result<String, reqwest::Error> {
            let body = self.client.get(url).send()?;
            body.text()
        }
        pub fn fetch_price(&self, sym: &str) -> Result<Price, Box<dyn Error>> { 
            let parse = |raw: &str| -> Result<Price, Box<dyn Error>> { 
                let v: Value = serde_json::from_str(&raw)?;
                let cprice = &v["market_data"]["current_price"];
                let in_eur: f32 = serde_json::from_value(cprice["eur"].to_owned())?;
                let in_usd: f32 = serde_json::from_value(cprice["usd"].to_owned())?;
                let time = &v["last_updated"];
                let last_updated: String = serde_json::from_value(time.to_owned())?;
                let format = "%Y-%m-%dT%H:%M:%S%.fZ";
                let dt = NaiveDateTime::parse_from_str(&last_updated, format)?;
                Ok(Price{dt, base: sym.to_string(), in_usd,in_eur})
            };
            let fetch = |n: &str| {
                let base_url = "https://api.coingecko.com/api/v3/coins/";
                let url = format!("{}{}", base_url, n);
                self.retrieve(&url)
            };
            let n = self.resolver.resolve(sym)?;
            let response = fetch(&n)?;
            let price = parse(&response)?;
            Ok(price)
        }

       pub fn fetch_historical_price(&self, sym: &str) -> Result<Vec<Price>, Box<dyn Error>> {
             let parse = |raw: &str| -> Result<Vec<Price>, Box<dyn Error>> { 
                let v: Value = serde_json::from_str(&raw)?;
                let prices: Vec<(i64,f32)> = from_value(v["prices"].to_owned())?;
                let ret = prices.iter().map(|p| {
                    let (ts, price) = p;
                    let seconds = ((ts.to_owned() as f32)/1000.0) as i64;
                    let dt = NaiveDateTime::from_timestamp(seconds, 0);
                    Price { dt, 
                        base: sym.to_owned(), 
                        in_eur: price.to_owned(),
                        in_usd: price.to_owned()
                    }
                }).collect();
                Ok(ret)
            };
            let fetch = |n: &str| { 
                let url = format!("https://api.coingecko.com/api/v3/coins/{}",n);
                let params = format!("/market_chart?vs_currency=eur&days=30&interval=hourly");
                let full_url = format!("{}{}", &url, &params);
                self.retrieve(&full_url)
            };
            let n = self.resolver.resolve(sym)?;
            let response = fetch(&n)?;
            let price = parse(&response)?;
            Ok(price)
       }

    }
}

