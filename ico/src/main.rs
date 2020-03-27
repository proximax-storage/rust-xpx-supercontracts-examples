extern crate serde;
extern crate xpx_supercontracts_sdk;

use std::fs;

use serde::Deserialize;

const CSV_FILE: &str = "/home/pc/Downloads/ico_init.csv";

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct CsvIcoData {
	#[serde(rename = "Name")]
	name: String,
	#[serde(rename = "Shareholder account address")]
	shareholder_address: String,
	#[serde(rename = "Amount")]
	amount: i64,
}

fn csv_parse(data: &Vec<u8>) -> Result<Vec<CsvIcoData>, std::io::Error> {
	let mut csv_reader = csv::ReaderBuilder::new()
		.delimiter(b',')
		.from_reader(&data[..]);

	let mut csv_result: Vec<CsvIcoData> = vec![];
	for res in csv_reader.deserialize() {
		let value: CsvIcoData = res.expect("can't parse");
		csv_result.push(value);
	}
	Ok(csv_result)
}

pub fn main() {
	let content = fs::read_to_string(&CSV_FILE.to_string()).expect("file not open");
	let res = csv_parse(&content.as_bytes().to_vec());
	println!("\n{:?}\n", res.unwrap());
}