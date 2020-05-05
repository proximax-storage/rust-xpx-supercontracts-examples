extern crate xpx_supercontracts_sdk;
extern crate csv;
extern crate serde;

use xpx_supercontracts_sdk::{
    storage::{storage_get, save_result},
    statuses:: {Result, Error},
    transactions_type::{FUNCTION_ERROR, FUNCTION_RETURN_SUCCESS},
    utils::debug_message,
};
use csv::{WriterBuilder};

const FIRST_MATRIX: &str = "matrixA.csv";
const SECOND_MATRIX: &str = "matrixB.csv";

#[no_mangle]
pub extern "C" fn multiple_matrices() -> i64 {
    let res = storage_get(&FIRST_MATRIX.to_string());
    if let Err(err) = res {
        debug_message(&format!("failed load CSV file: {:?}", err));
        return FUNCTION_ERROR;
    }

    let matrix_a = parse_matrix(&res.unwrap());
    if matrix_a.is_err() {
        return FUNCTION_ERROR;
    }
    let matrix_a = matrix_a.unwrap();

    let res = storage_get(&SECOND_MATRIX.to_string());
    if let Err(err) = res {
        debug_message(&format!("failed load CSV file: {:?}", err));
        return FUNCTION_ERROR;
    }

    let matrix_b = parse_matrix(&res.unwrap());
    if matrix_b.is_err() {
        return FUNCTION_ERROR;
    }
    let matrix_b = matrix_b.unwrap();

    if matrix_a[0].len() != matrix_b.len() {
        debug_message(&format!("matrices can't be multiplied"));
        return FUNCTION_ERROR;
    }

    let mut matrix_c:Vec<Vec<f64>> = Vec::new();

    for i in 0..matrix_a.len() {
        let mut vec = Vec::new();
        for j in 0..matrix_b[0].len() {
            let mut res = 0.0;
            for k in 0..matrix_b.len() {
                res += matrix_a[i][k] * matrix_b[k][j];
            }
            vec.push(res);
        }
        matrix_c.push(vec);
    }

    let res = match matrix_to_csv(matrix_c) {
        Ok(res) => res,
        Err(err) => {
            debug_message(&format!("can't convert matrix: {:?}", err));
            return FUNCTION_ERROR
        },
    };

    let res = save_result(&"result_matrix.txt".to_string(), res.as_slice());
    if let Err(err) = res {
        debug_message(&format!("failed save result: {:?}", err));
        return FUNCTION_ERROR;
    }

    FUNCTION_RETURN_SUCCESS
}

fn parse_matrix(data: &Vec<u8>) -> Result<Vec<Vec<f64>>> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(false)
        .from_reader(&data[..]);

    let mut matrix: Vec<Vec<f64>> = vec![];
    for res in rdr.deserialize() {
        if let Err(err) = res {
            debug_message(&format!("failed parse csv file: {:?}", err));
            return Err(Error::DeserializeJson);
        }

        let vec: Vec<f64> = res.unwrap();
        matrix.push(vec);
    }

    Ok(matrix)
}

fn matrix_to_csv(m:Vec<Vec<f64>>) -> Result<Vec<u8>> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(vec![]);

    for v in m {
        if let Err(err) = wtr.serialize(v) {
            debug_message(&format!("failed parse csv file: {:?}", err));
            return Err(Error::SerializeJson);
        }
    }

    let d =  wtr.into_inner();
    Ok(d.unwrap())
}
