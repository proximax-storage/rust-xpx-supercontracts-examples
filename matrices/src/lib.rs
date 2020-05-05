extern crate xpx_supercontracts_sdk;
extern crate csv;

use xpx_supercontracts_sdk::{
    storage::{storage_get, save_result},
    statuses:: {Result},
    transactions_type::FUNCTION_ERROR,
    utils::debug_message,
};
use xpx_supercontracts_sdk::transactions_type::FUNCTION_RETURN_SUCCESS;

use csv::{WriterBuilder};

pub extern "C" fn multiple_matrices(first: &String, second: &String) -> i64 {
    let matrix_a = storage_get(&first);
    let matrix_a = match matrix_a {
        Ok(matrix_a) =>
            match parse_matrix(matrix_a.as_ref()) {
                Ok(res) => res,
                Err(err) => {
                    debug_message(&format!("failed to read the first matrix: {:?}", err));
                    return FUNCTION_ERROR
                }
            }
        Err(err) => {
            debug_message(&format!("failed to get the first matrix: {:?}", err));
            return FUNCTION_ERROR
        }
    };

    let matrix_b = storage_get(&second);
    let matrix_b = match matrix_b {
        Ok(matrix_b) =>
            match parse_matrix(matrix_b.as_ref()) {
                Ok(res) => res,
                Err(err) => {
                    debug_message(&format!("failed to read the second matrix: {:?}", err));
                    return FUNCTION_ERROR
                },
            }
        Err(err) => {
            debug_message(&format!("failed to get the second matrix: {:?}", err));
            return FUNCTION_ERROR
        },
    };

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
        .from_reader(&data[..]);

    let mut matrix: Vec<Vec<f64>> = vec![];
    for res in rdr.deserialize() {
        let vec: Vec<f64> = res.expect("can't get row");
        matrix.push(vec);
    }

    Ok(matrix)
}

fn matrix_to_csv(m:Vec<Vec<f64>>) -> Result<Vec<u8>> {
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_writer(vec![]);

    for v in m {
        wtr.serialize(v);
    }

    let d =  wtr.into_inner();
    Ok(d.unwrap())
}
