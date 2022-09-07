// use crate::api::file::{ArchiveFormat, Extract, Move};

use crate::index::RemotePackage;
use crate::Error;

pub(crate) fn update(_package: RemotePackage) -> Result<(), Error> {
    // //Check for download

    // // We need an announced signature by the server
    // // if there is no signature, bail out.
    // verify_signature(&mut archive_buffer, &self.signature, &pub_key)?;

    // // Perform update
    // copy_files_and_run(archive_buffer, &self.extract_path)?;
    Ok(())
}

// fn copy_files_and_run<R: Read + Seek>(archive_buffer: R, extract_path: &Path) -> Result {
//     let mut extracted_files: Vec<PathBuf> = Vec::new();

//     // Extract the buffer to the tmp_dir
//     // we extract our signed archive into our final directory without any temp file
//     let mut extractor = Extract::from_cursor(archive_buffer,
// ArchiveFormat::Tar(Some(Compression::Gz)));     // the first file in the tar.gz will always be
//     // <app_name>/Contents
//     let tmp_dir = tempfile::Builder::new().prefix("fig_desktop").tempdir()?;

//     // create backup of our current app
//     Move::from_source(extract_path).to_dest(tmp_dir.path())?;

//     // extract all the files
//     extractor.with_files(|entry| {
//         let path = entry.path()?;
//         // skip the first folder (should be the app name)
//         let collected_path: PathBuf = path.iter().skip(1).collect();
//         let extraction_path = extract_path.join(collected_path);

//         // if something went wrong during the extraction, we should restore previous app
//         if let Err(err) = entry.extract(&extraction_path) {
//             for file in &extracted_files {
//                 // delete all the files we extracted
//                 if file.is_dir() {
//                     std::fs::remove_dir(file)?;
//                 } else {
//                     std::fs::remove_file(file)?;
//                 }
//             }
//             Move::from_source(tmp_dir.path()).to_dest(extract_path)?;
//             return Err(crate::api::Error::Extract(err.to_string()));
//         }

//         extracted_files.push(extraction_path);

//         Ok(false)
//     })?;

//     let _ = std::process::Command::new("touch").arg(&extract_path).status();

//     Ok(())
// }

// // Validate signature
// // need to be public because its been used
// // by our tests in the bundler
// //
// // NOTE: The buffer position is not reset.
// pub fn verify_signature<R>(archive_reader: &mut R, release_signature: &str, pub_key: &str) ->
// Result<bool> where
//     R: Read,
// {

// }
