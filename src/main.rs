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


const CONNECTION: &'static str = "wss://www.deribit.com/ws/api/v2";

#[throws(DeribitError)]
#[tokio::main]
async fn main() {
    // let stdout = stdout();
    // let mut stdout = stdout.lock();
    // let stdin = stdin();
    // let mut stdin = stdin.lock();
    //
    // loop {
    //     stdout.write_all(b"input: ").unwrap();
    //     stdout.flush().unwrap();
    //
    //     let input = stdin.read_line();
    //
    //     if let Ok(Some(input)) = input {
    //         let args: Vec<&str> = input.as_str().split_whitespace().collect();
    //         if args.len() == 0 {
    //             stdout.write_all("Please input something".as_bytes()).unwrap();
    //             stdout.write_all(b"\n").unwrap();
    //         } else {
    //             if let Err(e) = parse_stdin(args){
    //                 stdout.write_all(Error.to_string().as_bytes());
    //                 stdout.write_all(b"\n").unwrap();
    //             }
    //             sleep(Duration::from_millis(100));
    //         }
    //     } else {
    //         stdout.write_all(b"Error\n").unwrap();
    //     }
    // }
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


    let instruments = dqaunt::get_instruments().unwrap();

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

    let mut instr_state = dqaunt::Data{base: 0.0, three_months: 0.0, six_months: 0.0};

    let mut i = 0;

    let (tx, rx) = mpsc::channel();

    let thread = thread::spawn(move || {
        let recieved = rx.recv().unwrap();
        loop {
            println!("hi number {} from the spawned thread!", recieved);
            println!("Global var {:?}", dqaunt::global_state.lock().unwrap()s);
            thread::sleep(Duration::from_millis(1000));
        }

    });

    while let Some(m) = subscription.next().await {
        tx.send(i).unwrap();
        i +=1;
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
                    SubscriptionData::Ticker(ticker_data) => {dqaunt::get_expiration(ticker_data).unwrap()}
                }
            }
        }
    }
    thread.join().unwrap();

}