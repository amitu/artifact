extern crate artifact_app;

fn main() {
    match artifact_app::run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            ::std::process::exit(1);
        }
    }
}

