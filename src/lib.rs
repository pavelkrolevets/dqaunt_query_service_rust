#[macro_use] extern crate mysql;
use chrono::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
//use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Mul;
use deribit::models::{AuthRequest, Currency, GetPositionsRequest, PrivateSubscribeRequest, GetBookSummaryByInstrumentRequest, GetBookSummaryByCurrencyRequest};
use deribit::DeribitBuilder;
use deribit::DeribitError;
use dotenv::dotenv;
use futures::StreamExt;
use std::env::var;
use deribit::models::Currency::BTC;
use std::{thread, time};

const CONNECTION: &'static str = "wss://www.deribit.com/ws/api/v2";

#[derive(Debug, PartialEq)]
struct Data {
    base: i64,
    three_months: i64,
    six_months: i64,
}

#[derive(Debug, PartialEq)]
struct Instruments {
    pub id: u8,
    instrument_name: String,
    kind: String,
    expiration_timestamp: i64,
    is_active: bool,
    timestamp: DateTime<Utc>
}


enum Exchange {

}

//struct Vertex {
//    exchange: String,
//    currency: String
//}

pub fn parse_stdin(args: Vec<&str>) -> Result<(), DeribitError>{
    match args[0] {
        "start" => start(),
        "stop" => hello_world(args),
        _ => hello_world(args),
    };
    Ok(())
}

pub fn hello_world(args: Vec<&str>) -> Result<(), DeribitError> {
    println!("Args {:?}", args);
    Ok(())
}

//fn parse_update(args: Vec<&str>) -> Result<(), Error>{
//    // Parse timestamp
//    let mut timestamp = &args[0];
//    let date = DateTime::parse_from_rfc3339(&args[0] ).unwrap().timestamp();
//    println!("{}", date);
////    let timestamp = args[0].parse::<i64>().unwrap();
////    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
////    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
////    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
//
//
//    //Parse exchange and check
//    let exchange = args[1];
//    match exchange.to_lowercase().as_str() {
//        "gdax" => println!("Ok"),
//        "kraken" => println!("Ok"),
//        "fcoin" => println!("Ok"),
//        _ => panic!("Wrong exchange"),
//    }
//
//    // Parse currency and check
//    let source_currency = args[2];
//    match source_currency {
//        "BTC" => println!("Ok"),
//        "ETH" => println!("Ok"),
//        "USD" => println!("Ok"),
//        _ => panic!("Wrong currency"),
//    }
//
//    // Parse destination currency
//    let destination_currency = args[3];
//    match destination_currency {
//        "BTC" => println!("Ok"),
//        "ETH" => println!("Ok"),
//        "USD" => println!("Ok"),
//        _ => panic!("Wrong destination currency"),
//    }
//
//    let forward_factor = args[4].parse::<f64>().unwrap();
//    let backward_factor = args[5].parse::<f64>().unwrap();
//
//    if forward_factor * backward_factor > 1.0 {
//        panic!("Wrong exchange rates")
//    }
//
//    let upd = Rate::new(date, exchange,source_currency, destination_currency, forward_factor, backward_factor).unwrap();
//
//    let write = upd.write_to_db();
//
//    match write {
//        Ok(_)=> Ok(()),
//        Err(Error) => Err(Error)
//    }
//
//
//}

fn get_instruments() -> Result<(Vec<Instruments>), DeribitError>{
    println!("DB query ...");
    let pool = mysql::Pool::new("mysql://root:Gfdtk81,@localhost/deribit").unwrap();
    let instruments: Vec<Instruments> =
        pool.prep_exec(r"SELECT tt.* FROM instruments tt INNER JOIN (SELECT instrument_name, MAX(timestamp) AS MaxDateTime FROM instruments WHERE (is_active=:is_active_ AND kind=:kind_)) groupedtt  ON tt.timestamp = groupedtt.MaxDateTime", params! { "is_active_" => 1i8,
        "kind_"=> "future",
         })
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    // ⚠️ Note that from_row will panic if you don't follow your schema
                    let (id, instrument_name, kind, expiration_timestamp, is_active, timestamp) = mysql::from_row(row);
                    Instruments {
                        id: id,
                        instrument_name: instrument_name,
                        kind: kind,
                        expiration_timestamp: expiration_timestamp,
                        is_active: is_active,
                        timestamp: DateTime::from_utc(timestamp,Utc)
                    }
                }).collect()
            }).unwrap();

    println!("Instruments {:?}", instruments);

    Ok(instruments)
}



//impl Rate {
//    fn new(timestamp: i64, exchange: &str, source_currency: &str, destination_currency: &str, forward_factor: f64, backward_factor: f64)-> Result<Rate, Error>{
//        Ok(Rate {
//            timestamp: timestamp,
//            exchange: exchange.to_string(),
//            source_currency: source_currency.to_string(),
//            destination_currency: destination_currency.to_string(),
//            forward_factor: forward_factor,
//            backward_factor: backward_factor
//        })
//    }
//
//    fn write_to_db (&self)-> Result<(), Error> {
//        let pool = mysql::Pool::new("mysql://root:Gfdtk81,@localhost/tenx").unwrap();
//        for mut stmt in pool.prepare(r"INSERT INTO rates
//                                       (time, exchange, source_currency, destination_currency, forward_factor, backward_factor)
//                                   VALUES
//                                       (:time, :exchange, :source_currency, :destination_currency, :forward_factor, :backward_factor)").into_iter() {
//           stmt.execute(params! {
//                "time" => &self.timestamp,
//                "exchange" => &self.exchange,
//                "source_currency" => &self.source_currency,
//                "destination_currency" => &self.destination_currency,
//                "forward_factor" => &self.forward_factor,
//                "backward_factor" => &self.backward_factor,}).unwrap();
//        }
//        Ok(())
//    }
//}
#[tokio::main]
async fn start() -> Result<(), DeribitError> {
    println!("Connecting to {}", CONNECTION);

    let _ = dotenv();

    let key = "0RSVo90R".to_string();
    let secret = "T2FJDujLFttGUI-luTZ6AxYNIZ9sF14Jvegd3Unaeaw".to_string();

    let drb = DeribitBuilder::default().testnet(false).build().unwrap();

    let (mut client, mut subscription) = drb.connect().await?;

    let _ = client
        .call(AuthRequest::credential_auth(&key, &secret))
        .await?;

    let book = client
        .call(GetBookSummaryByCurrencyRequest::futures(BTC))
        .await?
        .await?;

    println!("{:?}", book);

    // loop {
    //     let book = client
    //         .call(GetBookSummaryByCurrencyRequest::futures(BTC))
    //         .await?
    //         .await?;
    //
    //     println!("{:?}", book);
    //
    //     thread::sleep(time::Duration::from_secs(5));
    // }

    // let instrument = "ETH-PERPETUAL".to_string();
    // let book = client
    //     .call(GetBookSummaryByInstrumentRequest::instrument(&instrument))
    //     .await?
    //     .await?;
    //
    // println!("{:?}", book);

    let instruments = get_instruments().unwrap();

    let mut channels = vec![];

    for item in instruments.iter(){
        let mut inst_str = "book.".to_string();
        inst_str.push_str(&item.instrument_name);
        inst_str.push_str(".100.1.100ms");

        &channels.insert(0, inst_str);
    }
    println!("Channels {:?}", &channels);

    let req = PrivateSubscribeRequest::new(&channels);

    let result = client.call(req).await?.await?;
    println!("Subscription result: {:?}", result);

    while let Some(sub) = subscription.next().await {
        println!("{:?}", sub);
    }

    Ok(())

}