use std::error::Error;
use deribit::models::subscription::TickerData;
use std::borrow::{Borrow, BorrowMut};
use std::ops::Deref;
use mysql::prelude::Queryable;
use std::sync::{Mutex, MutexGuard, Arc};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use deribit::models::{AuthRequest, GetPositionsRequest, Currency, GetInstrumentsRequest, GetInstrumentsResponse, AssetKind};
use deribit::{DeribitBuilder, DeribitError};
use mysql::Conn;


pub async fn write_instruments ()-> Result<(), Box<dyn Error>>{

    let key = "0RSVo90R".to_string();
    let secret = "T2FJDujLFttGUI-luTZ6AxYNIZ9sF14Jvegd3Unaeaw".to_string();

    let drb = DeribitBuilder::default().testnet(false).build().unwrap();
    let (mut client, _) = drb.connect().await?;
    let req = AuthRequest::credential_auth(&key, &secret);
    let _ = client.call(req).await?;

    let url = "mysql://root:Gfdtk81,@localhost/deribit";
    let pool = mysql::Pool::new(url)?;
    let mut mysql_conn = pool.get_conn()?.unwrap();
    let ping = mysql_conn.ping();

    if ping == false {
        println!("Cant connect to DB")
    } else {
        let instruments_btc = client.call(GetInstrumentsRequest::futures(Currency::BTC))
        .await?
        .await?;
        write_to_db(mysql_conn.borrow_mut(), &instruments_btc).unwrap();

        let instruments_eth = client.call(GetInstrumentsRequest::futures(Currency::ETH))
            .await?
            .await?;
        write_to_db(mysql_conn.borrow_mut(), &instruments_eth).unwrap();

    }

    // let _ = client
    //     .call(AuthRequest::credential_auth(&key, &secret))
    //     .await?;

    // let positions = client
    //     .call(GetPositionsRequest::futures(Currency::ETH))
    //     .await?
    //     .await?;
    //
    // println!("{:?}", positions);



    Ok(())
}

fn write_to_db (mysql_conn: &mut Conn, instruments: &Vec<GetInstrumentsResponse>) -> Result<(), Box<dyn Error>>{

    for i in instruments.iter() {
        let instr = GetInstrumentsResponse::from(i.clone());

        let instrument_name = instr.instrument_name;
        let kind = match instr.kind {
            AssetKind::Future => "future",
            AssetKind::Option => "option"
        };
        let expiration_timestamp = instr.expiration_timestamp;
        let is_active = instr.is_active;

        mysql_conn.exec_drop(r"INSERT INTO instruments (instrument_name, kind, expiration_timestamp, is_active)
                        VALUES(:instrument_name, :kind, :expiration_timestamp, :is_active)",
                       params! {
                        "instrument_name" => instrument_name,
                        "kind" => kind,
                        "expiration_timestamp" => expiration_timestamp,
                        "is_active" => is_active}
        )?;
    }

    Ok(())
}