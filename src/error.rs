use colored::Colorize;

/// Error
///

/// `eprintln` the error,
/// and then exit program with given code.
///
pub fn e_exit(e_type: &str, e_msg: &str, code: i32) -> ! {
    e_println(e_type, e_msg);
    std::process::exit(code);
}

/// `eprintln` the error,
///
pub fn e_println(e_type: &str, e_msg: &str) {
    eprintln!("{}", e_msg);
    eprintln!("{}<{}>: {}",
              "Error".red().bold(),
              e_type.yellow(),
              e_msg
    );
}

pub fn ok_println(tip: &str, msg: &str) {
    println!("{}<{}>: {}",
        "OK".green().bold(),
        tip.yellow(),
        msg
    );
}