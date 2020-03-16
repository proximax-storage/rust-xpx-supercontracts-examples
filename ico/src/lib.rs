extern crate serde;
extern crate xpx_supercontracts_sdk;

use serde::{Deserialize, Serialize};
use xpx_supercontracts_sdk::{
	statuses::{Error, Result},
	storage::{save_result, storage_get},
	transactions::{flush, get_mosaci_id, get_supercontract, mosaic_definition, transfer, get_initiator_pubkey},
	transactions_type::{Cid, FUNCTION_ERROR, FUNCTION_RETURN_SUCCESS, GetMosaicID, MosaicDefinition, MosaicProperties, SuperContract, Transfer},
	utils::{constructor, debug_message},
};

const ICO_CSV_FILE: &str = "ico_init.csv";
const ICO_MOSAIC_INFO: &str = "ico_info.json";
const TRANSACTIONS_LIMIT: u8 = 50;
const MOSAIC_NONCE: u32 = 0;

#[derive(Deserialize)]
struct CsvIcoData {
	#[allow(dead_code)]
	name: Option<String>,
	shareholder_address: String,
	amount: i64,
}

#[derive(Serialize, Deserialize,)]
struct IcoInfo {
	supercontract_id:  Cid,
	mosaic_id: i64,
}

pub fn create_ico() -> i64 {
	let mosaic_id = create_mosaic();
	if mosaic_id < FUNCTION_RETURN_SUCCESS {
		return mosaic_id;
	}

	let file_result = storage_get(&ICO_CSV_FILE.to_string());
	if let Err(err) = file_result {
		debug_message(&format!("failed load CSV file: {:?}", err));
		return FUNCTION_ERROR;
	}

	let csv_data = parse_csv(&file_result.unwrap());
	if csv_data.is_err() {
		return FUNCTION_ERROR;
	}

	let mut counter = 0;
	for data in csv_data.unwrap() {
		// Transfer tokens
		let res = transfer(&Transfer {
			pub_key: data.shareholder_address,
			amount: data.amount,
			asset_id: 1,
		});
		if let Err(err) = res {
			debug_message(&format!("failed transfer mosaic: {:?}", err));
			return FUNCTION_ERROR;
		}

		counter += 1;
		if counter >= TRANSACTIONS_LIMIT {
			let res = flush();
			if let Err(err) = res {
				debug_message(&format!("failed flush transaction: {:?}", err));
				return FUNCTION_ERROR;
			}
			counter = 0;
		}
	}

	FUNCTION_RETURN_SUCCESS
}

fn parse_csv(data: &Vec<u8>) -> Result<Vec<CsvIcoData>> {
	let mut csv_reader = csv::ReaderBuilder::new()
		.delimiter(b';')
		.from_reader(&data[..]);

	let mut csv_result: Vec<CsvIcoData> = vec![];
	for res in csv_reader.deserialize() {
		if let Err(err) = res {
			debug_message(&format!("failed parse csv file: {:?}", err));
			return Err(Error::DeserializeJson);
		}

		let value: CsvIcoData = res.unwrap();
		csv_result.push(value);
	}
	Ok(csv_result)
}

fn create_mosaic() -> i64 {
	let res = get_supercontract();
	if let Err(err) = res {
		debug_message(&format!("failed get supercontract: {:?}", err));
		return FUNCTION_ERROR;
	}

	let sc: SuperContract = res.unwrap();

	debug_message(&format!("SC.ID: {:?}", sc.id));
	let res = mosaic_definition(&MosaicDefinition {
		nonce: MOSAIC_NONCE,
		owner_public_key: sc.id.clone(),
		mosaic_props: Some(MosaicProperties {
			supply_mutable: true,
			transferable: true,
			divisibility: 0,
			optional_properties: vec![],
		}),
	});
	if let Err(err) = res {
		debug_message(&format!("failed create mosaic: {:?}", err));
		return FUNCTION_ERROR;
	}

	let result = res.unwrap();
	if result < FUNCTION_RETURN_SUCCESS {
		debug_message(&"failed create mosaic".to_string());
		return result;
	}

	let res = flush();
	if let Err(err) = res {
		debug_message(&format!("failed flush transaction: {:?}", err));
		return FUNCTION_ERROR;
	}

	let res = get_mosaci_id(&GetMosaicID {
		owner_public_key: sc.id.clone(),
		nonce: MOSAIC_NONCE,
	});
	if let Err(err) = res {
		debug_message(&format!("failed get mosaic_id: {:?}", err));
		return FUNCTION_ERROR;
	}

	let mosaic_id = res.unwrap();
	let res = save_mosaic_info(sc.id, mosaic_id);
	if res < FUNCTION_RETURN_SUCCESS {
		return FUNCTION_ERROR;
	}

	mosaic_id
}

fn save_mosaic_info(supercontract_id: Cid, mosaic_id: i64) -> i64 {
	let data_bytes = serde_json::to_vec(&IcoInfo {
		supercontract_id,
		mosaic_id,
	});
	if let Err(err) = data_bytes {
		debug_message(&format!("failed serialize: {:?}", err));
		return Error::SerializeJson as i64;
	}

	let data_bytes = data_bytes.unwrap();
	let res = save_result(&ICO_MOSAIC_INFO.to_string(), &data_bytes[..]);
	if let Err(err) = res {
		debug_message(&format!("failed save result: {:?}", err));
		return FUNCTION_ERROR;
	}

	FUNCTION_RETURN_SUCCESS
}

#[no_mangle]
pub extern "C" fn ico_init() -> i64 {
	constructor(create_ico)
}

#[no_mangle]
pub extern "C" fn send(param1: i64, param2: i64, param3: i64, param4: i64, param5: i64, amount: i64) -> i64 {
	let res = get_initiator_pubkey();
	if let Err(err) = res {
		debug_message(&format!("failed get_initiator_pubkey: {:?}", err));
		return FUNCTION_ERROR;		
	}
	let pk = res.unwrap();

	let mut bytes: Vec<u8> = vec![];
	let mut p1 = param1.to_le_bytes().to_vec();
	let mut p2 = param2.to_le_bytes().to_vec();
	let mut p3 = param3.to_le_bytes().to_vec();
	let mut p4 = param4.to_le_bytes().to_vec();
	let mut p5 = param5.to_le_bytes().to_vec();
	bytes.append(&mut p1);
	bytes.append(&mut p2);
	bytes.append(&mut p3);
	bytes.append(&mut p4);
	bytes.append(&mut p5);

	let res = std::str::from_utf8(&bytes);
	if let Err(err) = res {
		debug_message(&format!("failed convert address: {:?}", err));
		return FUNCTION_ERROR;
	}
	
	let address: &str = res.unwrap(); 
	debug_message(&format!("send [{:?}] to address: {:?}", amount, address));

	let res = storage_get(&ICO_MOSAIC_INFO.to_string());
	if let Err(err) = res {
		debug_message(&format!("failed transfer mosaic: {:?}", err));
		return FUNCTION_ERROR;
	}
	let res = res.unwrap();
	let res = serde_json::from_slice(&res[..]);
	if let Err(err) = res {
		debug_message(&format!("failed parse mosaic: {:?}", err));
		return FUNCTION_ERROR;
	}
	let mosaic_info: IcoInfo = res.unwrap();

	let res = get_supercontract();
	if let Err(err) = res {
		debug_message(&format!("failed parse ge SuperContract: {:?}", err));
		return FUNCTION_ERROR;
	}
	let sc = res.unwrap();
	
	// Check Token owaner
	if sc.drive.owner != pk {
		debug_message(&format!("failed send token from not owner's: {:?}", pk));
		return FUNCTION_ERROR;		
	}
	
	// Transfer tokens
	let res = transfer(&Transfer {
		pub_key: pk,
		amount: amount,
		asset_id: mosaic_info.mosaic_id as u64,
	});
	if let Err(err) = res {
		debug_message(&format!("failed transfer mosaic: {:?}", err));
		return FUNCTION_ERROR;
	}
	
	FUNCTION_RETURN_SUCCESS
}
