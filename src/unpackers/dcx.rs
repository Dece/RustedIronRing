use std::fs;
use std::io::{Read, Write};

use flate2::read::ZlibDecoder;
use nom::Err::{Error as NomError, Failure as NomFailure};

use crate::parsers::dcx;
use crate::unpackers::errors::{self as unpackers_errors, UnpackError};

pub fn extract_dcx(dcx_path: &str, output_path: &str) -> Result<(), UnpackError> {
    let mut dcx_file = fs::File::open(dcx_path)?;
    let file_len = dcx_file.metadata()?.len() as usize;
    let mut dcx_data = vec![0u8; file_len];
    dcx_file.read_exact(&mut dcx_data)?;
    let (data, dcx) = match dcx::parse(&dcx_data) {
        Ok(result) => { result }
        Err(NomError(e)) | Err(NomFailure(e)) => {
            let reason = unpackers_errors::get_nom_error_reason(e.1);
            return Err(UnpackError::Parsing("DCX parsing failed: ".to_owned() + &reason))
        }
        e => {
            return Err(UnpackError::Unknown(format!("Unknown error: {:?}", e)))
        }
    };

    let decomp_data = decompress_dcx(&dcx, data)?;

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(&decomp_data)?;
    Ok(())
}

fn decompress_dcx(dcx: &dcx::Dcx, comp_data: &[u8]) -> Result<Vec<u8>, UnpackError> {
    let method: &[u8] = dcx.params.method.as_slice();
    if method == b"DFLT" {
        decompress_dcx_deflate(dcx, comp_data)
    } else {
        let method_string = match std::str::from_utf8(method) {
            Ok(s) => { String::from(s) }
            Err(_) => { format!("{:?}", method) }
        };
        Err(UnpackError::Compression(format!("Unknown method: {}", method_string)))
    }
}

fn decompress_dcx_deflate(dcx: &dcx::Dcx, comp_data: &[u8]) -> Result<Vec<u8>, UnpackError> {
    let mut data = vec![0u8; dcx.sizes.uncompressed_size as usize];
    let mut deflater = ZlibDecoder::new(comp_data);
    deflater.read_exact(&mut data)?;
    Ok(data)
}
