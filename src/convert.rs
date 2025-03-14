use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

pub struct ConvertCombine;

const ONE_GB: u64 = 1_073_741_824; // 1GB = 1024^3 bytes
impl ConvertCombine {

    pub fn combine_all(paths: Vec<PathBuf>, output: PathBuf) {
        // 创建输出文件
        let mut output_file = match File::create(&output) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("无法创建输出文件 '{}': {}", output.display(), e);
                return;
            }
        };

        // 遍历所有输入路径
        for path in paths {
            // 打开输入文件
            let input_file = match File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("跳过文件 [{}]: 打开失败 - {}", path.display(), e);
                    continue;
                }
            };

            // 获取文件大小
            let metadata = match input_file.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("跳过文件 [{}]: 获取元数据失败 - {}", path.display(), e);
                    continue;
                }
            };

            // 根据文件大小选择处理策略
            if metadata.len() > ONE_GB {
                Self::process_large_file(input_file, &mut output_file, &path);
            } else {
                Self::process_small_file(input_file, &mut output_file, &path);
            }

            // 添加文件分隔符（保留原函数行为）
            if let Err(e) = writeln!(&mut output_file) {
                eprintln!("跳过文件 [{}]: 写入分隔符失败 - {}", path.display(), e);
                continue;
            }

            println!("成功合并文件: {}", path.display());
        }

        println!("\n合并完成！结果已保存到: {}", output.display());
    }

    /// 处理小文件（直接读取全部内容）
    fn process_small_file(input: File, output: &mut File, path: &PathBuf) {
        let mut contents = String::new();
        match BufReader::new(input).read_to_string(&mut contents) {
            Ok(_) => {
                if let Err(e) = write!(output, "{}", contents) {
                    eprintln!("写入失败 [{}]: {}", path.display(), e);
                }
            }
            Err(e) => eprintln!("读取失败 [{}]: {}", path.display(), e),
        }
    }

    /// 处理大文件（流式读写，8MB缓冲区）
    fn process_large_file(mut input: File, output: &mut File, path: &PathBuf) {
        let mut buffer = vec![0u8; 8 * 1024 * 1024]; // 8MB缓冲区
        let mut total = 0;

        loop {
            let bytes_read = match input.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => {
                    eprintln!("读取失败 [{}]: {}", path.display(), e);
                    break;
                }
            };

            match output.write_all(&buffer[..bytes_read]) {
                Ok(_) => total += bytes_read,
                Err(e) => {
                    eprintln!("写入失败 [{}]: {}", path.display(), e);
                    break;
                }
            }
        }

        println!("已流式传输 {} MB", total / 1024 / 1024);
    }
}