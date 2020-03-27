extern crate serde;
extern crate url;
extern crate xpx_supercontracts_sdk;

use std::collections::HashMap;

use serde::Deserialize;
use url::Url;
use xpx_supercontracts_sdk::{
	http::{http_get, HttpRequest},
	storage::storage_get,
	transactions_type::FUNCTION_ERROR,
	utils::debug_message,
};

const API_KEY_FILE: &str = "coinmarket.key";
const URL_LIST: &str = "https://sandbox-api.coinmarketcap.com/v1/cryptocurrency/listings/latest";

#[derive(Debug, Deserialize)]
struct MarketPriceStatus {
	timestamp: String,
	error_code: i64,
	error_message: Option<String>,
	elapsed: i64,
	credit_count: i64,
}

#[derive(Debug, Deserialize)]
struct MarketUSDData {
	price: f64,
	volume_24h: f64,
	percent_change_1h: f64,
	percent_change_24h: f64,
	percent_change_7d: f64,
	market_cap: f64,
	last_updated: String,
}

#[derive(Debug, Deserialize)]
struct MarketUSDQuote {
	#[serde(rename = "USD")]
	usd: MarketUSDData,
}

#[derive(Debug, Deserialize)]
struct MarketPriceData {
	id: i64,
	name: String,
	symbol: String,
	slug: String,
	num_market_pairs: i64,
	date_added: String,
	tags: Vec<String>,
	circulating_supply: f64,
	total_supply: f64,
	cmc_rank: i64,
	last_updated: String,
	quote: MarketUSDQuote,
}

#[derive(Debug, Deserialize)]
struct MarketPrice {
	status: MarketPriceStatus,
	data: Option<Vec<MarketPriceData>>,
}

#[no_mangle]
pub extern "C" fn get_market_info() -> i64 {
	let res = storage_get(&API_KEY_FILE.to_string());
	if let Err(err) = res {
		debug_message(&format!("failed read API KEY file: {:?}", err));
		return FUNCTION_ERROR;
	}

	let res = String::from_utf8(res.unwrap());
	if let Err(err) = res {
		debug_message(&format!("failed get API KEY: {:?}", err));
		return FUNCTION_ERROR;
	}
	let api_key = res.unwrap().replace("\n", "");

	let res = Url::parse(URL_LIST);
	if let Err(err) = res {
		debug_message(&format!("failed parse URL: {:?}", err));
		return FUNCTION_ERROR;
	}

	let mut base_url = res.unwrap();
	let start = 1;
	let limit = 3;
	let currency = "USD";
	base_url.query_pairs_mut()
		.append_pair("convert", currency)
		.append_pair("limit", format!("{}", limit).as_str())
		.append_pair("start", format!("{}", start).as_str());

	debug_message(&format!("Request URL: {:?}", base_url));

	let mut headers: HashMap<String, String> = HashMap::new();
	headers.insert("Accept".to_string(), "application/json".to_string());
	headers.insert("X-CMC_PRO_API_KEY".to_string(), api_key);

	let req = HttpRequest {
		url: base_url.to_string(),
		headers: headers,
	};
	let resp = http_get(&req);
	if let Err(err) = resp {
		debug_message(&format!("Request error: {:?}", err));
		return err as i64;
	}

	let content = resp.unwrap();

	let res = serde_json::from_slice(&content[..]);
	if let Err(err) = res {
		debug_message(&format!("failed parse response: {:?}", err));
		return FUNCTION_ERROR;
	}
	let info: MarketPrice = res.unwrap();
	if info.status.error_code != 0 {
		debug_message(&format!("Failed get market info: {}", info.status.error_message.unwrap()));
		return FUNCTION_ERROR;
	}

	debug_message(&format!("Market info: {:#?}", info));

	content.len() as i64
}
