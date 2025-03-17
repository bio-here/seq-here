use super::error::{e_exit, e_println, ok_println};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct ConvertCombine;

const LARGE_FILE_THRESHOLD: u64 = 1_073_741_824;
const CHUNK_SIZE: usize = 8 * 1024 * 1024;

impl ConvertCombine {

    pub fn combine_all(paths: Vec<PathBuf>, output: PathBuf) {

        let output = match File::create(&output) {
            Ok(f) => Arc::new(Mutex::new(f)),
            Err(e) => e_exit("FILE_CREATE", &format!("Failed to create file: {}", e), 1),
        };

        paths.par_iter().for_each(|path| {
            // Result handling closure
            let process_result = || -> std::io::Result<()> {
                let file = File::open(path)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, format!("Failed open: {}", e)))?;

                // Use memmap2
                let mmap = unsafe { Mmap::map(&file)? };

                if file.metadata()?.len() > LARGE_FILE_THRESHOLD {
                    Self::process_large(&mmap, &output)
                } else {
                    Self::process_small(&mmap, &output)
                }
            };

            match process_result() {
                Ok(_) => ok_println("Merge", &format!("{}", path.display()) ),
                Err(e) => e_println("PROCESS_ERROR", &format!("Failed to process file [{}]: {}", path.display(), e)),
            }
        });

        ok_println("MERGE_COMPLETE", "");
    }

    /// Small file: Directly write
    fn process_small(data: &Mmap, output: &Arc<Mutex<File>>) -> std::io::Result<()> {
        let mut writer = output.lock()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Lock failed: {}", e)))?;

        writer.write_all(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::WriteZero, format!("Write failed: {}", e)))?;

        Ok(())
    }

    /// Large file: Write by blocks
    fn process_large(data: &Mmap, output: &Arc<Mutex<File>>) -> std::io::Result<()> {
        let mut pos = 0;

        while pos < data.len() {
            let end = std::cmp::min(pos + CHUNK_SIZE, data.len());
            let chunk = &data[pos..end];

            let mut writer = output.lock()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Lock failed: {}", e)))?;

            writer.write_all(chunk)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::WriteZero, format!("Write by blocks failed: {}", e)))?;

            pos = end;
            ok_println("PROGRESS", &format!("Wrote already {} MB", pos / 1024 / 1024));
        }

        Ok(())
    }
}