extern crate url;
extern crate xpx_supercontracts_sdk;

use std::collections::HashMap;
use url::Url;

use xpx_supercontracts_sdk::{
    statuses::{Error, Result},
    storage::{save_result, storage_get},
    utils::debug_message,
    transactions_type::FUNCTION_ERROR,
}
use xpx_supercontracts_sdk::http::HttpRequest;

const API_KEY: &str = "coinmarket.key";

#[no_mangle]
pub extern "C" fn get_market_info() -> i64 {
    let res = storage_get(&API_KEY.to_string());
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

    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Accept".to_string(), "application/json".to_string());
    headers.insert("X-CMC_PRO_API_KEY".to_string(), api_key);
    
    let req = HttpRequest {
        url: "https://s.dou.ua/files/dou-200x200.png".to_string(),
        headers: headers,
    };
    let resp = http_get(&req);
    if let Err(err) = resp {
        // Return error status
        return err as i64;
    }
    // Return response body length
    resp.unwrap().len() as i64
}
