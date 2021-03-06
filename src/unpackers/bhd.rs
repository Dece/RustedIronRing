use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Seek, Write};
use std::path;

use nom::Err::{Error as NomError, Failure as NomFailure};

use crate::name_hashes;
use crate::formats::bhd;
use crate::unpackers::errors::UnpackError;
use crate::utils::fs as utils_fs;

/// Parse a BHD file and extract its content from sister BDT.
///
/// As names are often a path rather than a simple file name,
/// output path is used as the BHD root and required subdirs
/// are automatically created.
pub fn extract_bhd(
    bhd_path: &str,
    names: &HashMap<String, String>,
    output_path: &str
) -> Result<(), UnpackError> {
    let bhd_path = path::Path::new(bhd_path);
    let bhd_data = utils_fs::open_file_to_vec(bhd_path)?;
    let bhd = match bhd::parse(&bhd_data) {
        Ok((_, bhd)) => bhd,
        Err(NomError(e)) | Err(NomFailure(e)) => return Err(UnpackError::parsing_err("BHD", e.1)),
        e => return Err(UnpackError::Unknown(format!("Unknown error: {:?}", e)))
    };

    let bdt_path = bhd_path.to_path_buf().with_extension("bdt");
    let mut bdt_file = fs::File::open(bdt_path.to_str().unwrap())?;

    extract_files(&bhd, &mut bdt_file, &names, &output_path)?;
    Ok(())
}

/// Extract files from a BHD/BDT pair.
fn extract_files(
    bhd: &bhd::Bhd,
    bdt_file: &mut fs::File,
    names: &HashMap<String, String>,
    output_path: &str,
) -> Result<(), io::Error> {
    let output_path = path::Path::new(output_path);
    utils_fs::ensure_dir_exists(output_path)?;

    for bucket in &bhd.buckets {
        for entry in bucket {
            bdt_file.seek(io::SeekFrom::Start(entry.offset))?;
            let mut data = vec![0; entry.size as usize];
            bdt_file.read_exact(&mut data)?;

            let hash_str = name_hashes::hash_as_string(entry.hash);
            let rel_path: &str = match names.get(&hash_str) {
                Some(path) => {
                    path.trim_start_matches("/")
                }
                _ => {
                    eprintln!("No name for {}, using hash as name.", hash_str);
                    &hash_str
                }
            };
            let file_path = output_path.join(rel_path);
            utils_fs::ensure_dir_exists(file_path.parent().unwrap())?;
            let mut output_file = fs::File::create(file_path)?;
            output_file.write_all(&data)?;
        }
    }

    Ok(())
}
