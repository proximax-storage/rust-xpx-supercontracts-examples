use xpx_supercontracts_sdk::utils::constructor;

mod ico {
	use serde::Deserialize;

	use xpx_supercontracts_sdk::statuses::{Error, Result};
	use xpx_supercontracts_sdk::storage::storage_get;
	use xpx_supercontracts_sdk::transactions::{flush, get_supercontract, mosaic_definition, transfer};
	use xpx_supercontracts_sdk::transactions_type::{FUNCTION_ERROR, FUNCTION_RETURN_SUCCESS, MosaicDefinition, MosaicProperties, MosaicProperty, SuperContract, Transfer};
	use xpx_supercontracts_sdk::utils::debug_message;

	const ICO_CSV_FILE: &str = "ico_init.csv";
	const TRANSACTIONS_LIMIT: u8 = 50;

	#[derive(Debug, Deserialize)]
	struct CsvIcoData {
		name: Option<String>,
		shareholder_address: String,
		amount: i64,
	}

	pub fn create_ico() -> i64 {
		let mosaic_result = create_mosaic();
		if mosaic_result < FUNCTION_RETURN_SUCCESS {
			return mosaic_result;
		}

		let file_result = storage_get(&ICO_CSV_FILE.to_string());
		if let Err(err) = file_result {
			debug_message(&format!("failed load CSV file: {}", err));
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
					debug_message(&format!("failed flush transaction: {}", err));
					return FUNCTION_ERROR;
				}
				counter = 0;
			}
		}

		0
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
			nonce: 0,
			owner_public_key: sc.id,
			mosaic_props: Some(MosaicProperties {
				supply_mutable: true,
				transferable: true,
				divisibility: 0,
				optional_properties: vec![],
			}),
		});
		if let Err(err) = res {
			debug_message(&format!("failed create mosaic: {}", err));
			return FUNCTION_ERROR;
		}
		
		let result = res.unwrap();
		if result < FUNCTION_RETURN_SUCCESS {
			debug_message(&"failed create mosaic".to_string());
			return result;
		}

		let res = flush();
		if let Err(err) = res {
			debug_message(&format!("failed flush transaction: {}", err));
			return FUNCTION_ERROR;
		}

		FUNCTION_RETURN_SUCCESS
	}
}

#[no_mangle]
pub extern "C" fn ico_init() -> i64 {
	constructor(ico::create_ico)
}

