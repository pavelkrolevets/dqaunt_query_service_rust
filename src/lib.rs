#[macro_use] extern crate mysql;
use chrono::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
//use std::borrow::Borrow;
use std::collections::HashMap;
use std::ops::Mul;
use deribit::models::{AuthRequest, Currency, GetPositionsRequest, PrivateSubscribeRequest, GetBookSummaryByInstrumentRequest, GetBookSummaryByCurrencyRequest, PublicSubscribeRequest, SetHeartbeatRequest, SubscriptionParams, HeartbeatType, TestRequest, SubscribeResponse, TickerRequest, TickerResponse, GetInstrumentsRequest};
use deribit::DeribitBuilder;
use deribit::DeribitError;
use dotenv::dotenv;
use futures::StreamExt;
use std::env::var;
use deribit::models::Currency::{BTC, ETH};
use std::{thread, time};
use termion::event::Key::PageUp;
use std::thread::sleep;

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
        _ => start(),
    };
    Ok(())
}

pub fn hello_world(args: Vec<&str>) -> Result<(), DeribitError> {
    println!("Args {:?}", args);
    Ok(())
}

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

fn write_to_db (msg: SubscribeResponse) -> Result<(), DeribitError>{
    println!("{:?}", &msg);
    Ok(())
}


#[tokio::main]
async fn start() -> Result<(), DeribitError> {
    println!("Connecting to {}", CONNECTION);

    let _ = dotenv();

    let key = "0RSVo90R".to_string();
    let secret = "T2FJDujLFttGUI-luTZ6AxYNIZ9sF14Jvegd3Unaeaw".to_string();

    let drb = DeribitBuilder::default().testnet(false).build().unwrap();

    let (mut client, mut subscription) = drb.connect().await?;

    // let _ = client
    //     .call(AuthRequest::credential_auth(&key, &secret))
    //     .await?;

    // let positions = client
    //     .call(GetPositionsRequest::futures(Currency::BTC))
    //     .await?
    //     .await?;
    //
    // println!("{:?}", positions);
    //

    loop {
        let instruments = get_instruments().unwrap();
        for item in instruments.iter(){
            let ticker = client
                .call(TickerRequest::instrument(&item.instrument_name.as_str()))
                .await?
                .await?;
            println!("{:?}", ticker);
        }
        let freez = time::Duration::from_secs(5);
        sleep(freez)
    }


    // let instr = client
    //     .call_raw(GetInstrumentsRequest::options(ETH))
    //     .await?
    //     .await?;
    //
    // println!("{:?}", instr);

    // let book = client
    //     .call_raw(GetBookSummaryByInstrumentRequest::instrument("ETH-25SEP20".into()))
    //     .await?
    //     .await?;
    //
    // println!("{:?}", book);

    // let instruments = get_instruments().unwrap();
    // let mut channels = vec![];
    //
    // for item in instruments.iter(){
    //     let mut inst_str = "ticker.".to_string();
    //     inst_str.push_str(&item.instrument_name);
    //     inst_str.push_str(".100ms");
    //     &channels.insert(0, inst_str);
    // }
    // println!("Channels {:?}", &channels);
    //
    // let req = PublicSubscribeRequest::new(&channels);
    //
    // let _ = client.call(req).await?.await?;
    //
    // client
    //     .call(SetHeartbeatRequest::with_interval(30))
    //     .await?
    //     .await?;
    //
    // while let Some(m) = subscription.next().await {
    //
    //     // println!("{:?}", &m?.params);
    //     let msg = &m?.params;
    //
    //     if let SubscriptionParams::Heartbeat {
    //         r#type: HeartbeatType::TestRequest,
    //     } = &msg
    //     {
    //         client.call(TestRequest::default()).await?.await?;
    //     } else {
    //         // write_to_db(&msg);
    //         println!("{:?}", &msg);
    //     }
    //
    // }

    Ok(())

}