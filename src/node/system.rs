use crate::{Error, Config};

fn throw(error: Error) {
    println!("{}", error);
    std::process::exit(1);
}

pub fn spawn_thread<F: FnOnce(Config) -> Result<(), Error> + std::marker::Send + 'static>(func: F, config: Config) {
    std::thread::spawn(move|| {
        match func(config) {
            Ok(()) => (),
            Err(error) => throw(error),
        }
    });
}
