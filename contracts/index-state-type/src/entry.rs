use crate::error::Error;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, packed::*, prelude::*},
    high_level,
};
use core::result::Result;

const SUM_OF_INFO_CELLS: u8 = 12;
const INDEX_STATE_CELL_DATA_LEN: usize = 1 + 1; // index(u8) + length(u8)

pub fn main() -> Result<(), Error> {
    // update
    if check_type_script_exists_in_inputs()? {
        // Update the index state cell and the type scripts of input and output exist
        match check_cells_type_scripts_valid() {
            Ok(_) => check_index_state_cells_data(),
            Err(err) => Err(err),
        }
    }
    // Create the index state cell and the input type script doesn't exist
    else {
        load_output_type_script(|output_type_script| {
            let out_point = high_level::load_input_out_point(0, Source::Input)?;
            let type_args: Bytes = output_type_script.args().unpack();
            if &type_args[..] != out_point.as_slice() {
                return Err(Error::InvalidArgument);
            }
            let _ = check_index_state_cell_data(Source::GroupOutput)?;
            Ok(())
        })
    }
}


// Index state cell data: index(u8) | sum_of_time_info_cells(u8)
fn check_index_state_cell_data(source: Source) -> Result<Vec<u8>, Error> {
    let data = high_level::load_cell_data(0, source)?;
    if data.len() != INDEX_STATE_CELL_DATA_LEN {
        return Err(Error::IndexStateDataLenError);
    }
    if data[0] >= SUM_OF_INFO_CELLS {
        return Err(Error::IndexStateOutOfBound);
    }
    if data[1] != SUM_OF_INFO_CELLS {
        return Err(Error::InfoAmountError);
    }
    Ok(data)
}

fn check_index_state_cells_data() -> Result<(), Error> {
    let input_data = check_index_state_cell_data(Source::GroupInput)?;
    let output_data = check_index_state_cell_data(Source::GroupOutput)?;

    if input_data[0] == SUM_OF_INFO_CELLS - 1 {
        if output_data[0] != 0 {
            return Err(Error::IndexIncreaseError);
        }
    }
    else if input_data[0] + 1 != output_data[0] {
        return Err(Error::IndexIncreaseError);
    }
    Ok(())
}

fn load_output_type_script<F>(closure: F) -> Result<(), Error> where F: Fn(Script) -> Result<(), Error>, {
    match high_level::load_cell_type(0, Source::GroupOutput) {
        Ok(Some(output_type_script)) => closure(output_type_script),
        Ok(None) => Err(Error::IndexStateTypeNotExist),
        Err(_) => Err(Error::IndexStateTypeNotExist),
    }
}

fn check_cells_type_scripts_valid() -> Result<(), Error> {
    load_output_type_script(|_| match high_level::load_cell_type(0, Source::GroupInput) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(Error::IndexStateTypeNotExist),
        Err(_) => Err(Error::IndexStateTypeNotExist),
    })
}

fn check_type_script_exists_in_inputs() -> Result<bool, Error> {
    let script = high_level::load_script()?;
    let type_script_exists_in_inputs = high_level::QueryIter::new(high_level::load_cell_type, Source::Input).any(
        |type_script_opt| match type_script_opt {
            Some(type_script) => type_script.code_hash().raw_data()[..] == script.code_hash().raw_data()[..],
            None => false,
        },
    );
    Ok(type_script_exists_in_inputs)
}

