#[macro_use]
extern crate mysql;
extern crate lazy_static;
use chrono::*;
// use std::time::{SystemTime, UNIX_EPOCH, Duration};
// //use std::borrow::Borrow;
// use std::collections::HashMap;
// use std::ops::Mul;
use std::error::Error;
use deribit::models::subscription::TickerData;
use std::borrow::Borrow;
use std::ops::Deref;
use mysql::prelude::Queryable;
use lazy_static::lazy_static;
use std::sync::{Mutex, MutexGuard, Arc};
use std::collections::HashMap;
use std::sync::mpsc::Sender;

// lazy_static! {
//     pub static ref STATE: Mutex<HashMap<&'static str, f64>> = Mutex::new({
//         let mut m = HashMap::new();
//         m.insert("perpetual", 0f64);
//         m.insert("three", 0f64);
//         m.insert("six", 0f64);
//         m
//     });
//     // pub static ref global_state_btc: Mutex<HashMap<&'static str, f64>> = Mutex::new(HashMap::new());
// }

#[derive(PartialEq, Clone, Debug)]
pub enum Expiration {
    Base,
    Three,
    Six
}

#[derive(PartialEq, Clone, Debug)]
pub enum InstrumentType {
    BTC,
    ETH,
}

#[derive(Debug)]
pub struct Instruments {
    // pub id: u8,
    pub instrument_name: String,
    pub kind: String,
    pub expiration_timestamp: i64,
    pub is_active: bool,
    // pub timestamp: DateTime<Utc>
}

// pub fn parse_stdin(args: Vec<&str>) -> Result<(), Error>{
//     match args[0] {
//         "start" => start(),
//         "stop" => hello_world(args),
//         _ => start(),
//     };
//     Ok(())
// }

// pub fn hello_world(args: Vec<&str>) -> Result<(), Error> {
//     println!("Args {:?}", args);
//     Ok(())
// }

// pub fn get_global_state<'a>() -> MutexGuard<'a, HashMap<&'static str, f64>> {
//     STATE.lock().unwrap()
// }

pub async fn get_instruments() -> Result<Vec<Instruments>, Box<dyn Error>>{
    // println!("DB query ...");

    let url = "mysql://root:Gfdtk81,@localhost/deribit";
    let pool = mysql::Pool::new(url)?;

    let mut conn = pool.get_conn()?.unwrap();

    let instruments =
        conn.query_map(r"SELECT tt.* FROM instruments tt INNER JOIN (SELECT instrument_name, MAX(timestamp) AS MaxDateTime FROM instruments WHERE (is_active=TRUE AND kind='future')) groupedtt  ON tt.timestamp = groupedtt.MaxDateTime", |(instrument_name, kind, expiration_timestamp, is_active)| {
            Instruments {
                // id: id,
                instrument_name: instrument_name,
                kind: kind,
                expiration_timestamp: expiration_timestamp,
                is_active: is_active,
                // timestamp: DateTime::from_utc(timestamp, Utc)
            }
        })?;

    Ok(instruments)
}

pub fn get_timestamp (timestamp: i64) -> Result<DateTime<Utc>, Box<dyn Error>> {
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    Ok(datetime)
}

pub fn get_expiration(msg: TickerData, instr: &Vec<Instruments>, data: &Arc<Mutex<HashMap<&'static str, f64>>>) -> Result<(), Box<dyn Error>>{

    // let instr = get_instruments().unwrap();
    let mut instr_btc= vec![];
    let mut instr_eth= vec![];

    for item in instr.iter(){
        if item.instrument_name[..3] == "BTC".to_string() {
            instr_btc.insert(0, item.clone())
        }
        else if &item.instrument_name[..3] == "ETH".to_string() {
            instr_eth.insert(0, item.clone())
        }
    }

    // Sort by timestamp
    // &instr_btc.sort_by(|a, b| b.expiration_timestamp.cmp(&a.expiration_timestamp));
    // &instr_eth.sort_by(|a, b| b.expiration_timestamp.cmp(&a.expiration_timestamp));
    //
    // println!("BTC instruments_db by timestamp {:?}", &instr_btc);
    // println!("ETH instruments_db by timestamp {:?}", &instr_eth);

    let mut exp = Expiration::Base;
    let mut currency = InstrumentType::BTC;

    if msg.instrument_name[..3] == "BTC".to_string() {
        currency = InstrumentType::BTC;
        for item in instr_btc.iter() {
            if (item.instrument_name == msg.instrument_name) && (item.instrument_name != "BTC-PERPETUAL".to_string()) {
                // println!("Found {:?}, expiration {:?}", &msg.instrument_name, &item.expiration_timestamp);

                // let msg_exp = get_timestamp(item.expiration_timestamp).unwrap();
                // let newdate = msg_exp.format("%Y-%m-%d %H:%M:%S").to_string();
                // println!("Found {:?}, datetime {:?}", &msg.instrument_name, &newdate);

                // find expiration of the future
                for i in instr_btc.iter() {
                    if i.instrument_name != "BTC-PERPETUAL".to_string() {
                        // let i_exp = get_timestamp(i.expiration_timestamp).unwrap();
                        if item.expiration_timestamp > i.expiration_timestamp {
                            exp = Expiration::Six
                        } else if item.expiration_timestamp < i.expiration_timestamp {
                            exp = Expiration::Three
                        }
                    }
                }
            } else if (item.instrument_name == msg.instrument_name) && (msg.instrument_name == "BTC-PERPETUAL".to_string()) {
                exp = Expiration::Base
            }
        }
    }

    if msg.instrument_name[..3] == "ETH".to_string() {
        currency = InstrumentType::ETH;
        for item in instr_eth.iter() {
            if (item.instrument_name == msg.instrument_name) && (item.instrument_name != "ETH-PERPETUAL".to_string()) {
                // println!("Found {:?}, expiration {:?}", &msg.instrument_name, &item.expiration_timestamp);

                // let msg_exp = get_timestamp(item.expiration_timestamp).unwrap();
                // let newdate = msg_exp.format("%Y-%m-%d %H:%M:%S").to_string();
                // println!("Found {:?}, datetime {:?}", &msg.instrument_name, &newdate);

                // find expiration of the futures
                for i in instr_eth.iter() {
                    if i.instrument_name != "ETH-PERPETUAL".to_string() {
                        // let i_exp = get_timestamp(i.expiration_timestamp).unwrap();
                        if item.expiration_timestamp > i.expiration_timestamp {
                            // println!("Here {:?}, {:?}, {:?}", currency, exp, msg.instrument_name);
                            exp = Expiration::Six
                        } else if item.expiration_timestamp < i.expiration_timestamp {
                            exp = Expiration::Three
                        }
                    }
                }
            } else if (item.instrument_name == msg.instrument_name) && (item.instrument_name == "ETH-PERPETUAL".to_string()){
                exp = Expiration::Base
            }
        }
    }

    // write_to_db(msg, exp, currency).unwrap();
    // println!("Currency {:?}", currency);
    // println!("Expiration {:?}", exp);
    match currency {
        InstrumentType::BTC => {
            match exp {
                Expiration::Base => {

                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"btc_perpetual").unwrap() = msg.last_price.unwrap();


                    // if let Some(x) = STATE.lock().unwrap().get_mut(&"perpetual") {
                    //     *x = msg.last_price.unwrap();
                    // }

                    println!("Insert BTC Perpetual {:?}, last {:?}, timestamp {:?}", &msg.instrument_name, &msg.last_price.unwrap(), &msg.timestamp)
                },
                Expiration::Three=>{

                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"btc_three").unwrap() = msg.last_price.unwrap();

                    // if let Some(x) = STATE.lock().unwrap().get_mut(&"three") {
                    //     *x = msg.last_price.unwrap();
                    // }

                    // println!("Insert BTC Three {:?}, last {:?}", &msg.instrument_name, &msg.last_price.unwrap())
                },
                Expiration::Six  => {

                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"btc_six").unwrap() = msg.last_price.unwrap();

                    // if let Some(x) = STATE.lock().unwrap().get_mut(&"six") {
                    //     *x = msg.last_price.unwrap();
                    // }

                    // println!("Insert BTC Six {:?}", &msg.instrument_name)
                }
            }
        }
        InstrumentType::ETH => {
            match exp {
                Expiration::Base => {
                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"eth_perpetual").unwrap() = msg.last_price.unwrap();
                    // println!("Insert ETH Perpetual {:?}, last {:?}", &msg.instrument_name, &msg.last_price.unwrap())

                },
                Expiration::Three => {
                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"eth_three").unwrap() = msg.last_price.unwrap();
                    // println!("Insert ETH Three {:?}, last {:?}", &msg.instrument_name, &msg.last_price.unwrap())
                },
                Expiration::Six => {
                    let mut x = data.lock().unwrap();
                    *x.get_mut(&"eth_six").unwrap() = msg.last_price.unwrap();
                    // println!("Insert EHT Six {:?}", &msg.instrument_name)
                }
            }
        }
    }


    // let state = get_global_state();
    // println!("Global var {:?} {:?} {:?}", &state.get(&"perpetual"),
    //                           &state.get(&"three"),
    //                           &state.get(&"six")
    //                  );

    Ok(())
}

pub fn write_to_db (data: &Arc<Mutex<HashMap<&'static str, f64>>>)->  Result<(), Box<dyn Error>> {

    let url = "mysql://root:Gfdtk81,@localhost/deribit";
    let pool = mysql::Pool::new(url)?;

    // let pool = mysql::Pool::new("mysql://root:Gfdtk81,@localhost/deribit").unwrap();

    let mut conn = pool.get_conn()?.unwrap();
    let ping = conn.ping();

    // println!("Connected to db {:?}", ping);
    let x = data.lock().unwrap();
    let btc_perp = *x.get("btc_perpetual").unwrap();
    let btc_three = *x.get("btc_three").unwrap();
    let btc_six = *x.get("btc_six").unwrap();

    conn.exec_drop(r"INSERT INTO futures_contango_btc (perpetual, three_months, six_months)
      VALUES(:perpetual, :three_months, :six_months)",
              params! {
                "perpetual" => btc_perp,
                "three_months" => btc_three,
                "six_months" => btc_six,
    }
    )?;

    let eth_perp = *x.get("eth_perpetual").unwrap();
    let eth_three = *x.get("eth_three").unwrap();
    let eth_six = *x.get("eth_six").unwrap();

    conn.exec_drop(r"INSERT INTO futures_contango_eth (perpetual, three_months, six_months)
      VALUES(:perpetual, :three_months, :six_months)",
                   params! {
                "perpetual" => eth_perp,
                "three_months" => eth_three,
                "six_months" => eth_six,
    }
    )?;

    Ok(())

}