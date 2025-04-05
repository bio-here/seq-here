use super::error::{e_exit, e_println, ok_println};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Handles conversion and combining operations for files
pub struct ConvertCombine;

const LARGE_FILE_THRESHOLD: u64 = 1_073_741_824; // 1GB - threshold for using chunked processing
const CHUNK_SIZE: usize = 16 * 1024 * 1024; // 16MB - size of each processing chunk
const PROGRESS_INTERVAL: Duration = Duration::from_secs(1); // Report progress once per second

impl ConvertCombine {

    /// Combines multiple files into a single output file
    /// 
    /// # Arguments
    /// 
    /// * `paths` - Vector of paths to input files
    /// * `output` - Path to the output file
    pub fn combine_all(paths: Vec<PathBuf>, output: PathBuf) {
        // Use BufWriter to improve write performance
        let output_file = match File::create(&output) {
            Ok(f) => f,
            Err(e) => e_exit("FILE_CREATE", &format!("Failed to create file: {}", e), 1),
        };
        let buffered_output = BufWriter::with_capacity(8 * 1024 * 1024, output_file);
        let output = Arc::new(Mutex::new(buffered_output));

        // Add progress tracking
        let total_processed = Arc::new(Mutex::new(0u64));

        paths.par_iter().for_each(|path| {
            // Result handling closure
            let process_result = || -> std::io::Result<()> {
                let file = File::open(path)?;
                let file_size = file.metadata()?.len();

                // Use memmap2
                let mmap = unsafe { Mmap::map(&file)? };

                if file_size > LARGE_FILE_THRESHOLD {
                    Self::process_large(&mmap, &output, &total_processed)
                } else {
                    Self::process_small(&mmap, &output)
                }
            };

            match process_result() {
                Ok(_) => ok_println("Merge", &format!("{}", path.display())),
                Err(e) => e_println("PROCESS_ERROR", &format!("Failed to process file [{}]: {}", path.display(), e)),
            }
        });

        // Ensure all data is flushed to disk
        if let Ok(mut writer) = output.lock() {
            let _ = writer.flush();
        }

        ok_println("MERGE_COMPLETE", "");
    }

    /// Process a small file by writing it directly to the output
    /// 
    /// # Arguments
    /// 
    /// * `data` - Memory-mapped file data
    /// * `output` - Shared output writer
    fn process_small(data: &Mmap, output: &Arc<Mutex<BufWriter<File>>>) -> std::io::Result<()> {
        let mut writer = output.lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire lock"))?;

        writer.write_all(data)?;
        Ok(())
    }

    /// Process a large file in chunks with progress reporting
    /// 
    /// # Arguments
    /// 
    /// * `data` - Memory-mapped file data
    /// * `output` - Shared output writer
    /// * `total_processed` - Counter for total bytes processed
    fn process_large(
        data: &Mmap, 
        output: &Arc<Mutex<BufWriter<File>>>,
        total_processed: &Arc<Mutex<u64>>
    ) -> std::io::Result<()> {
        let mut pos = 0;
        let mut last_report_time = Instant::now();
        let mut last_report_pos = 0;

        while pos < data.len() {
            // Determine the end position for the current chunk
            let end = std::cmp::min(pos + CHUNK_SIZE, data.len());
            let chunk = &data[pos..end];

            // Acquire lock and write chunk
            {
                let mut writer = output.lock()
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Failed to acquire lock"))?;
                writer.write_all(chunk)?;
            }

            pos = end;
            
            // Reduce progress reporting frequency based on time interval rather than per chunk
            let now = Instant::now();
            if now.duration_since(last_report_time) >= PROGRESS_INTERVAL {
                let bytes_written = pos - last_report_pos;
                let elapsed = now.duration_since(last_report_time).as_secs_f64();
                let mb_per_sec = (bytes_written as f64 / 1_048_576.0) / elapsed;
                
                // Update total processed bytes
                if let Ok(mut total) = total_processed.lock() {
                    *total += bytes_written as u64;
                }
                
                ok_println("PROGRESS", &format!("Processed {:.2} MB, Speed: {:.2} MB/s", 
                    pos as f64 / 1_048_576.0, mb_per_sec));
                
                last_report_time = now;
                last_report_pos = pos;
            }
        }

        Ok(())
    }
}
