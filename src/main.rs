use std::collections::HashMap;
use std::env;
use std::option::Option;
use std::process;
use reqwest::blocking::Client;
use reqwest::header;
use serde::Deserialize;
use serde_json;
use serde_json::json;
use chrono::{prelude::*, Duration};



#[derive(Debug)]
struct State {
    client_id: String,
    client_secret: String,
    bearer_token: Option<String>,
    client: Option<Client>,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    scope: String,
    created_at: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut creds = State {
        client_id: env::var("CLIENT_ID").expect("Missing 'CLIENT_ID' env var"),
        client_secret: env::var("CLIENT_SECRET").expect("Missing 'CLIENT_SECRET' env var"),
        bearer_token: None,
        client: None,
    };


    if !do_auth(&mut creds) {
        process::exit(1);
    }

    let custom_investments = get_custom_investments(&creds);
    
    println!("custom_investments = #{:?}", custom_investments);

    let (date, price) = get_vanguard_global_small_cap_index_fund_price();

    assert_eq!(1, custom_investments.len());
    
    let ci = &custom_investments[0];
    if add_custom_investment_price(&creds, ci, price.into(), date) {
        println!("Success");
        Ok(())
    } else {
        eprintln!("Failed to put custom price");
        Err("Fail".into())
    }
}

#[derive(Deserialize, Debug)]
struct CustomInvestments {
    custom_investments: Vec<CustomInvestment>,
}

#[derive(Deserialize, Debug)]
struct CustomInvestment {
    code: String,
    market_code: String,
    name: String,
    id: u32,
}


// https://www.vanguardinvestor.co.uk/investments/vanguard-global-small-cap-index-fund-gbp-acc/price-performance
// api call this pages makes is:
// $ curl "https://api.vanguard.com/rs/gre/gra/1.7.0//datasets/urd-product-port-specific-price-history.json?vars=portId%3A9158%2CissueType%3AS%2CstartDate%3A2022-01-03%2CendDate%3A2022-01-13&callback=angular.callbacks._j"
// [{"date":"2022-01-12T00:00:00-05:00","navPrice":398.2687,"mmNavPrice":"398.2687"},{"date":"2022-01-11T00:00:00-05:00","navPrice":398.982,"mmNavPrice":"398.9820"},{"date":"2022-01-10T00:00:00-05:00","navPrice":396.7258,"mmNavPrice":"396.7258"},{"date":"2022-01-07T00:00:00-05:00","navPrice":399.3158,"mmNavPrice":"399.3158"},{"date":"2022-01-06T00:00:00-05:00","navPrice":402.9598,"mmNavPrice":"402.9598"},{"date":"2022-01-05T00:00:00-05:00","navPrice":403.6885,"mmNavPrice":"403.6885"},{"date":"2022-01-04T00:00:00-05:00","navPrice":410.9371,"mmNavPrice":"410.9371"}]
#[derive(Deserialize, Debug)]
struct HistoricVanguardFundPrice {
    date: String,
    #[serde(rename = "navPrice")]
    price: f64,
}

fn get_vanguard_global_small_cap_index_fund_price() -> (String, f64) {
    let today = Local::now().format("%Y-%m-%d").to_string();
    let ten_ago = (Utc::now() - Duration::days(10)).format("%Y-%m-%d").to_string();
    let url = format!("https://api.vanguard.com/rs/gre/gra/1.7.0//datasets/urd-product-port-specific-price-history.json?vars=portId%3A9158%2CissueType%3AS%2CstartDate%3A{}%2CendDate%3A{}", ten_ago, today);
    // println!("{}", url);
    let prices: Vec<HistoricVanguardFundPrice> =
        reqwest::blocking::get(url).unwrap().json().unwrap();
    
    let hp = &prices[0];
    let mut date = hp.date.clone();
    date.truncate(10);
    println!("Vanguard small-cap price {} = {}", date, hp.price);
    (date, hp.price)
}

fn get_custom_investments(creds: &State) -> Vec<CustomInvestment> {
    let h: CustomInvestments =
        creds.client.as_ref().unwrap()
        .get("https://api.sharesight.com/api/v3/custom_investments.json")
        .send()
        .unwrap()
        .json()
        .unwrap()
        ;
    h.custom_investments
}

fn add_custom_investment_price(creds: &State, ci: &CustomInvestment, price: f64, date: String) -> bool {
    println!("Setting custom investment price for {} to {} @ {}", ci.name, price, date);
    let mut body: HashMap::<String, serde_json::Value> = HashMap::new();
    body.insert("last_traded_on".into(), json!(date));
    body.insert("last_traded_price".into(), json!(price));

    // let url = format!("http://localhost:9999/api/v3/prices/{}.json", ci.id);
    let url = format!("https://api.sharesight.com/api/v3/custom_investment/{}/prices.json", ci.id);
    // println!("url = {} body = {:?}", url, body);
    let r = creds.client.as_ref().unwrap().post(url).json(&body).send().unwrap();
    r.status().is_success()
}

fn do_auth(creds: &mut State) -> bool {
    let params = [
        ("grant_type", "client_credentials"),
        ("client_id", &creds.client_id),
        ("client_secret", &creds.client_secret)
    ];

    let client = reqwest::blocking::Client::new();
    let resp = client.post("https://api.sharesight.com/oauth2/token")
        .form(&params)
        .send()
        .expect("Error during http auth")
        ;
    if !resp.status().is_success() {
        println!("Error during auth {:#?}", resp);
        false
    } else {
        let j: AuthResponse = resp.json().expect("Invalid auth response json");
        creds.bearer_token = Some(j.access_token.clone());

        
        let mut headers = header::HeaderMap::new();
        let mut auth_val = header::HeaderValue::from_str(
            format!("Bearer {}", j.access_token.clone()).as_str()).unwrap();
        auth_val.set_sensitive(true);
        headers.insert("Authorization", auth_val);

        // println!("Authorization: Bearer {}", j.access_token.clone());

        let client = reqwest::blocking::Client::builder()
        .user_agent("RJ's tool for updating custom investment price")
        .default_headers(headers)
        .build()
        .expect("Couldn't build http client")
        ;
        creds.client = Some(client);

        true
    }
}