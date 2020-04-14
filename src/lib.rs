#[macro_use] extern crate mysql;
use chrono::*;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fmt::Error;
//use std::borrow::Borrow;
use std::collections::HashMap;
use petgraph::graphmap::DiGraphMap;
use std::ops::Mul;

#[derive(Debug, PartialEq)]
struct Rate {
    pub timestamp: i64,
    pub exchange: String,
    source_currency: String,
    destination_currency: String,
    forward_factor: f64,
    backward_factor: f64
}

enum Exchange {

}

//struct Vertex {
//    exchange: String,
//    currency: String
//}

pub fn parse_stdin(args: Vec<&str>) -> Result<(), Error>{

    match args[0] {
        "EXCHANGE_RATE_REQUEST" => parse_request(args),
        _ => parse_update(args)
    };
    Ok(())
}

fn parse_update(args: Vec<&str>) -> Result<(), Error>{
    // Parse timestamp
    let mut timestamp = &args[0];
    let date = DateTime::parse_from_rfc3339(&args[0] ).unwrap().timestamp();
    println!("{}", date);
//    let timestamp = args[0].parse::<i64>().unwrap();
//    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
//    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
//    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");


    //Parse exchange and check
    let exchange = args[1];
    match exchange.to_lowercase().as_str() {
        "gdax" => println!("Ok"),
        "kraken" => println!("Ok"),
        "fcoin" => println!("Ok"),
        _ => panic!("Wrong exchange"),
    }

    // Parse currency and check
    let source_currency = args[2];
    match source_currency {
        "BTC" => println!("Ok"),
        "ETH" => println!("Ok"),
        "USD" => println!("Ok"),
        _ => panic!("Wrong currency"),
    }

    // Parse destination currency
    let destination_currency = args[3];
    match destination_currency {
        "BTC" => println!("Ok"),
        "ETH" => println!("Ok"),
        "USD" => println!("Ok"),
        _ => panic!("Wrong destination currency"),
    }

    let forward_factor = args[4].parse::<f64>().unwrap();
    let backward_factor = args[5].parse::<f64>().unwrap();

    if forward_factor * backward_factor > 1.0 {
        panic!("Wrong exchange rates")
    }

    let upd = Rate::new(date, exchange,source_currency, destination_currency, forward_factor, backward_factor).unwrap();

    let write = upd.write_to_db();

    match write {
        Ok(_)=> Ok(()),
        Err(Error) => Err(Error)
    }


}

fn parse_request(args: Vec<&str>) -> Result<(), Error>{
    let pool = mysql::Pool::new("mysql://root:Gfdtk81,@localhost/tenx").unwrap();
    let rates: Vec<Rate> =
        pool.prep_exec(r"SELECT tt.* FROM rates tt INNER JOIN (SELECT exchange, MAX(time) AS MaxDateTime FROM rates WHERE (source_currency=:s_currency AND destination_currency=:d_currency) OR (source_currency=:d_currency AND destination_currency=:s_currency)) groupedtt  ON tt.time = groupedtt.MaxDateTime", params! { "s_currency" => &args[2].to_string(),
        "d_currency"=>&args[4].to_string(),
         })
            .map(|result| {
                result.map(|x| x.unwrap()).map(|row| {
                    // ⚠️ Note that from_row will panic if you don't follow your schema
                    let (date, exchange,source_currency, destination_currency, forward_factor, backward_factor) = mysql::from_row(row);
                    Rate {
                        timestamp: date,
                        exchange: exchange,
                        source_currency: source_currency,
                        destination_currency: destination_currency,
                        forward_factor: forward_factor,
                        backward_factor: backward_factor
                    }
                }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
            }).unwrap(); // Unwrap `Vec<Payment>`

    println!("Exchange rates {:?}", rates);
    let mut V = vec![];

    for i in rates.iter() {
        V.insert(0, (i.exchange.as_str(), i.source_currency.as_str()));
        V.insert(0, (i.exchange.as_str(), i.destination_currency.as_str()));
    }

    println!("V {:?}", V);

    let mut rate = DiGraphMap::new();
    let mut next = DiGraphMap::new();

    for item in rates.iter() {
        rate.add_edge((item.exchange.as_str(), item.source_currency.as_str()), (item.exchange.as_str(), item.destination_currency.as_str()), item.forward_factor);
        next.add_edge((item.exchange.as_str(), item.source_currency.as_str()), (item.exchange.as_str(), item.destination_currency.as_str()), (item.exchange.as_str(), item.destination_currency.as_str()));
        rate.add_edge((item.exchange.as_str(), item.destination_currency.as_str()), (item.exchange.as_str(), item.source_currency.as_str()), item.backward_factor);
        next.add_edge((item.exchange.as_str(), item.destination_currency.as_str()), (item.exchange.as_str(), item.source_currency.as_str()), (item.exchange.as_str(), item.source_currency.as_str()));
    }
    for i in rates.iter() {
        for j in rates.iter() {
            rate.add_edge((i.exchange.as_str(), i.source_currency.as_str()), (j.exchange.as_str(), j.source_currency.as_str()), 1f64);
            next.add_edge((i.exchange.as_str(), i.source_currency.as_str()), (j.exchange.as_str(), j.source_currency.as_str()), (j.exchange.as_str(), j.source_currency.as_str()));
            rate.add_edge((i.exchange.as_str(), i.destination_currency.as_str()), (j.exchange.as_str(), j.destination_currency.as_str()), 1f64);
            next.add_edge((i.exchange.as_str(), i.destination_currency.as_str()), (j.exchange.as_str(), j.destination_currency.as_str()),(j.exchange.as_str(), j.destination_currency.as_str()));
        }
    }


    println!("Graph {:?}", rate.edge_count());
    println!("Edge weight {:?}", rate.edge_weight( ("FCOIN", "BTC"), ("FCOIN", "USD")));
    println!("Edge weight {:?}", rate.edge_weight( ("FCOIN", "USD"), ("FCOIN", "BTC")));
    println!("Edge weight {:?}", rate.edge_weight( ("GDAX", "BTC"), ("FCOIN", "BTC")));

    for edge in rate.all_edges(){
        println!("Rate {:?}",edge)
    }
    for edge in next.all_edges(){
        println!("Next {:?}",edge)
    }

    for k in V.iter(){
       for i in V.iter(){
           for j in V.iter(){
//                println!("V .. {:?}, {:?}, {:?}", i,j,k);
//               println!("Result {:?}", rate.edge_weight(*i, *k).unwrap_or(&1f64));
                let mut z:&mut f64 = &mut (rate.edge_weight(*i, *k).unwrap_or(&1f64) * rate.edge_weight(*k, *j).unwrap_or(&1f64));
               if rate.edge_weight(*i, *j).unwrap_or(&1f64) < z  {
                   println!("Floyd Warshall work... {:?}", z);
                   let mut weight = rate.edge_weight_mut(*i, *j);
                    weight = Some(z);
                   println!("New weight ... {:?}", rate.edge_weight(*i, *j));
               }
           }
       }
    }

    Ok(())
}



impl Rate {
    fn new(timestamp: i64, exchange: &str, source_currency: &str, destination_currency: &str, forward_factor: f64, backward_factor: f64)-> Result<Rate, Error>{
        Ok(Rate {
            timestamp: timestamp,
            exchange: exchange.to_string(),
            source_currency: source_currency.to_string(),
            destination_currency: destination_currency.to_string(),
            forward_factor: forward_factor,
            backward_factor: backward_factor
        })
    }

    fn write_to_db (&self)-> Result<(), Error> {
        let pool = mysql::Pool::new("mysql://root:Gfdtk81,@localhost/tenx").unwrap();
        for mut stmt in pool.prepare(r"INSERT INTO rates
                                       (time, exchange, source_currency, destination_currency, forward_factor, backward_factor)
                                   VALUES
                                       (:time, :exchange, :source_currency, :destination_currency, :forward_factor, :backward_factor)").into_iter() {
           stmt.execute(params! {
                "time" => &self.timestamp,
                "exchange" => &self.exchange,
                "source_currency" => &self.source_currency,
                "destination_currency" => &self.destination_currency,
                "forward_factor" => &self.forward_factor,
                "backward_factor" => &self.backward_factor,}).unwrap();
        }
        Ok(())
    }
}