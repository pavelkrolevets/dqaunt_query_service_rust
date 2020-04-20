extern crate termion;
extern crate dqaunt;
#[macro_use] extern crate mysql;

use std::fmt::Error;
use termion::input::TermRead;
use std::io::{Write, stdout, stdin};
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use deribit::models::{AuthRequest, Currency, GetPositionsRequest, PrivateSubscribeRequest, GetBookSummaryByInstrumentRequest, GetBookSummaryByCurrencyRequest, PublicSubscribeRequest, SetHeartbeatRequest, SubscriptionParams, HeartbeatType, TestRequest, SubscribeResponse, TickerRequest, TickerResponse, GetInstrumentsRequest, GetInstrumentsResponse, AssetKind};
use deribit::DeribitBuilder;
use deribit::DeribitError;
use deribit::models::{SubscriptionData};
use dotenv::dotenv;
use futures::{StreamExt, TryFutureExt};
use std::env::var;
use deribit::models::Currency::{BTC, ETH};
use std::{thread, time};
use termion::event::Key::PageUp;
use env_logger::init;
use fehler::throws;
use std::sync::mpsc;
use serde_json::error::Category::Data;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, Arc};
use dqaunt::Instruments;
use chrono::DateTime;
use chrono::*;
use mysql::error::DriverError::NamedParamsForPositionalQuery;
mod instruments_db {pub mod write_instr;}


const CONNECTION: &'static str = "wss://www.deribit.com/ws/api/v2";

#[throws(DeribitError)]
#[tokio::main]
async fn main() {

    let _ = dotenv();
    init();
    println!("Connecting to {}", CONNECTION);

    let drb = DeribitBuilder::default().testnet(false).build().unwrap();

    let (mut client, mut subscription) = drb.connect().await?;

    // let write_instr = instruments_db::write_instr::write_instruments().await;
    //
    // match write_instr {
    //     Ok(()) => println!("Instruments written to db."),
    //     Err(e)=> panic!("Cant write instruments_db to DB: {:?}", e)
    // };

    let instruments_btc = client.call(GetInstrumentsRequest::futures(Currency::BTC))
        .await?
        .await?;

    let instruments_eth = client.call(GetInstrumentsRequest::futures(Currency::ETH))
        .await?
        .await?;

    let mut instruments:Vec<Instruments> = vec![];

    for item in instruments_btc.iter(){
        let i = GetInstrumentsResponse::from(item.clone());

        let instr_kind = match i.kind {
            AssetKind::Future => "future".to_string(),
            AssetKind::Option => "option".to_string()
        };

        let instr = Instruments {
            // id: 0u8,
            instrument_name: i.instrument_name,
            kind: instr_kind,
            expiration_timestamp: i.expiration_timestamp as i64,
            is_active: i.is_active,

        };
        instruments.push(instr);
    }

    for item in instruments_eth.iter(){
        let i = GetInstrumentsResponse::from(item.clone());

        let instr_kind = match i.kind {
            AssetKind::Future => "future".to_string(),
            AssetKind::Option => "option".to_string()
        };

        let instr = Instruments {
            // id: 0u8,
            instrument_name: i.instrument_name,
            kind: instr_kind,
            expiration_timestamp: i.expiration_timestamp as i64,
            is_active: i.is_active,

        };
        instruments.push(instr);
    }


    // let instruments = dqaunt::get_instruments().await;
    //
    println!("instruments {:?}", &instruments);

    // let instruments = match instruments{
    //     Ok(i) => i,
    //     Err(e)=> panic!("Cant get instruments_db from DB: {:?}", e)
    // };

    let mut channels = vec![];

    for item in instruments.iter(){
        let mut inst_str = "ticker.".to_string();
        inst_str.push_str(&item.instrument_name);
        inst_str.push_str(".100ms");
        &channels.insert(0, inst_str);
    }
    println!("Channels {:?}", &channels);

    let req = PublicSubscribeRequest::new(&channels);

    let _ = client.call(req).await?.await?;

    // client
    //     .call(SetHeartbeatRequest::with_interval(30))
    //     .await?
    //     .await?;

    let data = Arc::new(Mutex::new({
        let mut m = HashMap::new();
        m.insert("btc_perpetual", 0f64);
        m.insert("btc_three", 0f64);
        m.insert("btc_six", 0f64);
        m.insert("eth_perpetual", 0f64);
        m.insert("eth_three", 0f64);
        m.insert("eth_six", 0f64);
        m
    })
    );

    let data_read = data.clone();

    let thread = thread::spawn(move || {
        loop {

            let wr_db = dqaunt::write_to_db(&data_read).unwrap();

            // match wr_db {
            //     Ok(()) => println!("Written to db."),
            //     Err(e)=> panic!("Cant write to DB: {:?}", e)
            // };

            println!("Result: {:?}", &data_read.lock().unwrap());
            thread::sleep(Duration::from_millis(5000));
        }

    });

    // let thread = thread::spawn(move || {
    //     loop {
    //
    //         let wr_db = instruments_db::write_instr::write_instruments();
    //
    //         match wr_db {
    //             Ok(()) => println!("Written to db."),
    //             Err(e)=> panic!("Cant write to DB: {:?}", e)
    //         };
    //
    //         thread::sleep(Duration::from_millis(30000));
    //     }
    // });
    let data_mut = data.clone();

    while let Some(m) = subscription.next().await {

        // let mut base_guard = data_mut.lock().unwrap();
        // *base_guard.base += 1f64;


        match m?.params {
            SubscriptionParams::Heartbeat {r#type} => if let r#type = HeartbeatType::TestRequest {
                println!("Hartbeat {:?}", r#type);
            }
            SubscriptionParams::Subscription {channel, data} => {
                match data {
                    SubscriptionData::Announcements(AnnouncementsData) => (),
                    SubscriptionData::Book(BookData) => (),
                    SubscriptionData::DeribitPriceIndex(DeribitPriceIndexData) => (),
                    SubscriptionData::DeribitPriceRanking(DeribitPriceRankingData) => (),
                    SubscriptionData::EstimatedExpirationPrice(EstimatedExpirationPriceData) => (),
                    SubscriptionData::GroupedBook(GroupedBookData) => (),
                    SubscriptionData::MarkPriceOption(MarkPriceOptionData) => (),
                    SubscriptionData::Perpetual(PerpetualData) => (),
                    SubscriptionData::Quote(QuoteData) => (),
                    SubscriptionData::Trades(TradesData) => (),
                    SubscriptionData::UserOrders(UserOrdersData) => (),
                    SubscriptionData::UserOrdersBatch(UserOrdersData) => (),
                    SubscriptionData::UserPortfolio(UserPortfolioData) => (),
                    SubscriptionData::UserTrades(UserTradesData) => (),
                    SubscriptionData::Ticker(ticker_data) => dqaunt::get_expiration(ticker_data, &instruments, &data_mut).unwrap()
                }
            }
        }
    }

    thread.join().unwrap();
    // write_instr_thread.join().unwrap()

}