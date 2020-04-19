extern crate termion;
extern crate dqaunt;

use std::fmt::Error;
use termion::input::TermRead;
use std::io::{Write, stdout, stdin};
use std::thread::sleep;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use deribit::models::{AuthRequest, Currency, GetPositionsRequest, PrivateSubscribeRequest, GetBookSummaryByInstrumentRequest, GetBookSummaryByCurrencyRequest, PublicSubscribeRequest, SetHeartbeatRequest, SubscriptionParams, HeartbeatType, TestRequest, SubscribeResponse, TickerRequest, TickerResponse, GetInstrumentsRequest};
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


const CONNECTION: &'static str = "wss://www.deribit.com/ws/api/v2";

#[throws(DeribitError)]
#[tokio::main]
async fn main() {

    let _ = dotenv();
    init();
    println!("Connecting to {}", CONNECTION);

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

    // loop {
    //     let instruments = get_instruments().unwrap();
    //     for item in instruments.iter(){
    //         let ticker = client
    //             .call(TickerRequest::instrument(&item.instrument_name.as_str()))
    //             .await?
    //             .await?;
    //         println!("{:?}", ticker);
    //     }
    //     let freez = time::Duration::from_secs(5);
    //     sleep(freez)
    // }


    let instruments = dqaunt::get_instruments();

    let instruments = match instruments {
        Ok(i) => i,
        Err(e)=> panic!("Cant get instruments from DB: {:?}", e)
    };

    let mut channels = vec![];

    for item in instruments.iter(){
        let mut inst_str = "ticker.".to_string();
        inst_str.push_str(&item.instrument_name);
        inst_str.push_str(".100ms");
        &channels.insert(0, inst_str);
    }
    // println!("Channels {:?}", &channels);

    let req = PublicSubscribeRequest::new(&channels);

    let _ = client.call(req).await?.await?;

    client
        .call(SetHeartbeatRequest::with_interval(30))
        .await?
        .await?;

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

    let data_mut = data.clone();

    let data_read = data.clone();

    let thread = thread::spawn(move || {
        loop {
            // let recieved: Option<&f64> = rx.recv().unwrap();
            println!("Result: {:?}", *data.lock().unwrap());

            let wr_db = dqaunt::write_to_db(&data_read);

            match wr_db {
                Ok(()) => println!("Written to db."),
                Err(e)=> panic!("Cant write to DB: {:?}", e)
            };

            thread::sleep(Duration::from_millis(5000));
        }

    });

    // let instr = dqaunt::get_instruments();
    //
    // let instr = match instr {
    //     Ok(i) => i,
    //     Err(e)=> panic!("Cant get instruments from DB: {:?}", e)
    // };

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
                    SubscriptionData::Ticker(ticker_data) => {dqaunt::get_expiration(ticker_data, &instruments, &data_mut).unwrap()}
                }
            }
        }
    }

    thread.join().unwrap();

}