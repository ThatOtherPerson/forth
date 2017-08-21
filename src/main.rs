extern crate forth;

use std::io::{self, Write};
use forth::Runtime;

fn main() {
    let mut forth = Runtime::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        match forth.eval(&buffer) {
            Ok(_) => println!("ok"),
            Err(e) => eprintln!("! {}", e),
        }
    }
}
