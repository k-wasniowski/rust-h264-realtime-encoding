use log::info;
use simple_logger::SimpleLogger;

pub fn initialize() {
    match SimpleLogger::new().init() {
        Ok(_) => info!("SimpleLogger initialized!"),
        Err(_) => panic!("Failed to initialize simple logger")
    };
}