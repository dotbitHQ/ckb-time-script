use crate::error::Error;
use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, ckb_types::packed::*, high_level};
use common::constants::*;
use core::result::Result;

pub fn main() -> Result<(), Error> {
    // update
    if check_type_script_exists(Source::Input)? {
        if !(check_type_script_exists(Source::Output)?) {
            // Cells can be recycled
            Ok(())
        } else {
            // Update the index state cell and the type scripts of input and output exist
            match check_cells_type_scripts_valid() {
                Ok(_) => check_index_state_cells_data(),
                Err(err) => Err(err),
            }
        }
    }
    // Create the index state cell and the input type script doesn't exist
    else {
        load_output_type_script(|_| {
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
    // Due to the need for dynamic resizing SUM_OF_INFO_CELLS, do not check the data of input IndexStateCell anymore.
    // let input_data = check_index_state_cell_data(Source::GroupInput)?;

    let input_data = high_level::load_cell_data(0, Source::GroupInput)?;
    let output_data = check_index_state_cell_data(Source::GroupOutput)?;

    if input_data[0] >= SUM_OF_INFO_CELLS - 1 {
        if output_data[0] != 0 {
            return Err(Error::IndexIncreaseError);
        }
    } else if input_data[0] + 1 != output_data[0] {
        return Err(Error::IndexIncreaseError);
    }
    Ok(())
}

fn load_output_type_script<F>(closure: F) -> Result<(), Error>
where
    F: Fn(Script) -> Result<(), Error>,
{
    match high_level::load_cell_type(0, Source::GroupOutput) {
        Ok(Some(output_type_script)) => closure(output_type_script),
        Ok(None) => Err(Error::IndexStateTypeNotExist),
        Err(_) => Err(Error::IndexStateTypeNotExist),
    }
}

fn check_cells_type_scripts_valid() -> Result<(), Error> {
    load_output_type_script(
        |_| match high_level::load_cell_type(0, Source::GroupInput) {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(Error::IndexStateTypeNotExist),
            Err(_) => Err(Error::IndexStateTypeNotExist),
        },
    )
}

fn check_type_script_exists(source: Source) -> Result<bool, Error> {
    let script = high_level::load_script()?;
    let type_script_exists =
        high_level::QueryIter::new(high_level::load_cell_type, source).any(|type_script_opt| {
            match type_script_opt {
                Some(type_script) => {
                    type_script.code_hash().raw_data()[..] == script.code_hash().raw_data()[..]
                }
                None => false,
            }
        });
    Ok(type_script_exists)
}
