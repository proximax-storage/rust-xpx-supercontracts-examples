extern crate serde;
extern crate url;
extern crate xpx_supercontracts_sdk;

use std::collections::HashMap;

use serde::Deserialize;
use url::Url;
use xpx_supercontracts_sdk::{
	http::{http_get, HttpRequest},
	transactions_type::FUNCTION_ERROR,
	utils::debug_message,
	storage::storage_get
};

const API_KEY_FILE: &str = "coinmarket.key";
const URL_LIST: &str = "https://sandbox-api.coinmarketcap.com/v1/cryptocurrency/listings/latest";

#[derive(Debug, Deserialize)]
struct MarketPriceStruct {
	timestamp: String,
	error_code: i64,
	error_message: Option<String>,
	elapsed: i64,
	credit_count: i64,
}

#[derive(Debug, Deserialize)]
struct MarketPrice {
	status: MarketPriceStruct,
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
	let api_key = res.unwrap();

	let res = Url::parse(URL_LIST);
	if let Err(err) = res {
		debug_message(&format!("failed parse URL: {:?}", err));
		return FUNCTION_ERROR;
	}
	let mut base_url = res.unwrap();
	let start = 1;
	let limit = 1;
	let currency = "USD";
	base_url.set_query(Some(&format!("start={}", start)));
	base_url.set_query(Some(&format!("limit={}", limit)));
	base_url.set_query(Some(&format!("currency={}", currency)));

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
	debug_message(&format!("Request content: {:?}", content));

	let res = serde_json::from_slice(&content[..]);
	if let Err(err) = res {
		debug_message(&format!("failed parse response: {:?}", err));
		return FUNCTION_ERROR;
	}
	let info: MarketPrice = res.unwrap();
	debug_message(&format!("Market info: {:#?}", info));

	content.len() as i64
}
