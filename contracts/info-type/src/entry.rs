use crate::error::Error;
use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, packed::*, prelude::*},
    high_level::{load_cell_data, load_cell_type, load_input_since, load_script, QueryIter},
};
use common::constants::*;
use core::result::Result;

pub fn main() -> Result<(), Error> {
    // update info cell
    if check_type_script_exists_in_inputs()? {
        // Update the info cell and the info type scripts of input and output exist
        match check_cells_type_scripts_valid() {
            Ok(_) => check_info_cells_data(),
            Err(err) => Err(err),
        }
    }
    // Create the info cell and the input info type script doesn't exist
    else {
        load_output_type_script(|output_type_script| {
            let index_state_type_args = load_output_index_state_type_args()?;
            let info_type_args: Bytes = output_type_script.args().unpack();
            if info_type_args[..] != index_state_type_args[..] {
                return Err(Error::InvalidArgument);
            }
            check_info_cell_data()
        })
    }
}

fn load_output_index_state_type_args() -> Result<Bytes, Error> {
    match load_cell_type(0, Source::Output) {
        Ok(Some(output_type_script)) => {
            let type_args: Bytes = output_type_script.args().unpack();
            Ok(type_args)
        }
        Ok(None) => Err(Error::IndexStateTypeNotExist),
        Err(_) => Err(Error::IndexStateTypeNotExist),
    }
}

// Info cell data: index(u8) | type(u8) | DataType(u64)
fn check_info_cell_data() -> Result<(), Error> {
    match load_cell_data(0, Source::GroupOutput) {
        Ok(info_data) => match is_info_data_len_valid(&info_data, &info_data) {
            true => Ok(()),
            false => Err(Error::InfoDataLenError),
        },
        Err(_) => Err(Error::InfoTypeNotExist),
    }
}

fn check_info_cells_data() -> Result<(), Error> {
    // Note: Assuming that the first output must have index state type
    let output_index_state_data = load_cell_data(0, Source::Output)?;
    if output_index_state_data.len() != INDEX_STATE_CELL_DATA_LEN {
        return Err(Error::IndexStateDataLenError);
    }

    let input_info_data = load_cell_data(0, Source::GroupInput)?;
    let output_info_data = load_cell_data(0, Source::GroupOutput)?;

    if !is_info_data_len_valid(&input_info_data, &output_info_data) {
        return Err(Error::InfoDataLenError);
    }

    // the first u8 is index
    if output_info_data[0] != output_index_state_data[0] {
        return Err(Error::InfoIndexNotSame);
    }

    let since = load_input_since(0, Source::GroupInput)?;
    let output_info_data_type = output_info_data[INFO_CELL_META_TYPE_POS];

    // the second u8 is DataType
    if output_info_data_type == DataType::Timestamp as u8 {
        let input_timestamp = content_from_info_data(&input_info_data);
        let output_timestamp = content_from_info_data(&output_info_data);

        if input_timestamp >= output_timestamp {
            return Err(Error::OutputTimestampNotBigger);
        }

        let since_timestamp_base: u64 = 1 << 62;
        if since_timestamp_base + output_timestamp as u64 != since {
            return Err(Error::InvalidTimeInfoSince);
        }
    } else if output_info_data_type == DataType::BlockNumber as u8 {
        let input_block_number = content_from_info_data(&input_info_data);
        let output_block_number = content_from_info_data(&output_info_data);

        if input_block_number >= output_block_number {
            return Err(Error::OutputBlockNumberNotBigger);
        }

        if output_block_number != since {
            return Err(Error::InvalidTimeInfoSince);
        }
    } else {
        /* nothing here, maybe more validations in the future */
    }

    Ok(())
}

fn content_from_info_data(info_data: &Vec<u8>) -> u64 {
    let mut content_buf = [0u8; INFO_CELL_CONTENT_LEN];
    content_buf.copy_from_slice(&info_data[2..]);
    u64::from_be_bytes(content_buf)
}

fn is_info_data_len_valid(input_info_data: &Vec<u8>, output_info_data: &Vec<u8>) -> bool {
    input_info_data.len() == INFO_CELL_DATA_LEN && output_info_data.len() == INFO_CELL_DATA_LEN
}

fn load_output_type_script<F>(closure: F) -> Result<(), Error>
where
    F: Fn(Script) -> Result<(), Error>,
{
    match load_cell_type(0, Source::GroupOutput) {
        Ok(Some(output_type_script)) => closure(output_type_script),
        Ok(None) => Err(Error::InfoTypeNotExist),
        Err(_) => Err(Error::InfoTypeNotExist),
    }
}

fn check_cells_type_scripts_valid() -> Result<(), Error> {
    load_output_type_script(|_| match load_cell_type(0, Source::GroupInput) {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(Error::InfoTypeNotExist),
        Err(_) => Err(Error::InfoTypeNotExist),
    })
}

fn check_type_script_exists_in_inputs() -> Result<bool, Error> {
    let script = load_script()?;
    let type_script_exists_in_inputs = QueryIter::new(load_cell_type, Source::Input).any(
        |type_script_opt| match type_script_opt {
            Some(type_script) => {
                type_script.code_hash().raw_data()[..] == script.code_hash().raw_data()[..]
            }
            None => false,
        },
    );
    Ok(type_script_exists_in_inputs)
}
