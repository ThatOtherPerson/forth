use std::io::{self, Write};
use std::collections::HashMap;

enum Word {
    Native(fn(&mut Runtime)),
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

    fn eval(&mut self, source: &str) {
        for name in source.split_whitespace() {
            self.eval_name(name);
        }

        println!("ok");
    }

    fn eval_name(&mut self, name: &str) {
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

    fn eval_as_number(&mut self, name: &str) {
        if let Ok(number) = name.parse() {
            self.stack.push(number);
        } else {
            eprintln!("Undefined word");
            // TODO: return error
            //   so that the main REPL doesn't print "ok"
        }
    }

    fn eval_value(&mut self, value: &Word) {
        match value {
            &Word::Number(num) => println!("not actually sure what to do with this... {}", num),
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

fn rt_forth_print(forth: &mut Runtime) {
    println!("{}", forth.pop().expect("Stack underflow"));
}

fn main() {
    let mut forth = Runtime::new();

    let rfp: fn(&mut Runtime) = rt_forth_print;

    forth.register(".", Word::Native(rfp));

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();

        forth.eval(&buffer);
    }
}
