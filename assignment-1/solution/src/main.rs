use dotenv::dotenv;
use std::{env, path::Path};
use tp1::{cafeteria, error_dispenser::ErrorCafeteria, utils::init_logger};

fn main() -> Result<(), ErrorCafeteria> {
    init_logger();
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let mut file_name = &String::from("orders.txt"); // default file name

    if args.len() > 1 {
        file_name = &args[1];
    }

    cafeteria::start(Path::new(file_name))
}
