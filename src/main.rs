use std::io::{self, Write};
use std::collections::HashMap;

enum Word {
    Native(fn(&mut Runtime) -> Result<(), String>),
    Number(i32) // is this a thing, in Forth?
}

impl Clone for Word {
    fn clone(&self) -> Self {
        match self {
            &Word::Native(callback) => Word::Native(callback),
            &Word::Number(num) => Word::Number(num)
        }
    }
}

struct Runtime<'a> {
    dictionary: HashMap<&'a str, Word>,
    stack: Vec<i32>
}

impl <'a> Runtime<'a> {
    fn new() -> Runtime<'a> {
        Runtime {
            dictionary: HashMap::new(),
            stack: vec![]
        }
    }

    fn eval(&mut self, source: &str) -> Result<(), String> {
        for name in source.split_whitespace() {
            let result = self.eval_name(name);

            if let Err(_) = result {
                return result;
            }
        }

        Ok(())
    }

    // TODO: alias result type (FResult?)
    fn eval_name(&mut self, name: &str) -> Result<(), String> {
        let word = {
            let dict = &self.dictionary;
            dict.get(name).map(|w| { w.clone() })
        };
        if let Some(value) = word {
            self.eval_value(&value)
        } else {
            self.eval_as_number(name)
        }
    }

    fn eval_as_number(&mut self, name: &str) -> Result<(), String> {
        match name.parse() {
            Ok(num) => {
                self.stack.push(num);
                Ok(())
            },
            Err(_) => Err("Undefined word".to_string())
            // TODO: more descriptive error type
        }
    }

    fn eval_value(&mut self, value: &Word) -> Result<(), String> {
        match value {
            &Word::Number(num) => Err(format!("not actually sure what to do with this... {}", num)),
            &Word::Native(callback) => callback(self)
        }
    }

    fn register(&mut self, name: &'a str, word: Word) {
        self.dictionary.insert(name, word);
    }

    // TODO: maybe the forth type should be an alias or something
    fn pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }
}

fn rt_forth_print(forth: &mut Runtime) -> Result<(), String> {
    let popped = forth.pop();

    match popped {
        Some(value) => {
            println!("{}", value);
            Ok(())
        },
        None => Err("Stack underflow".to_string())
    }
}

fn main() {
    let mut forth = Runtime::new();

    let rfp: fn(&mut Runtime) -> Result<(), String> = rt_forth_print;

    forth.register(".", Word::Native(rfp));

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        match forth.eval(&buffer) {
            Ok(_) => println!("ok"),
            Err(e) => eprintln!("! {}", e)
        }
    }
}
