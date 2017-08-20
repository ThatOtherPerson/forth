use std::collections::{VecDeque, HashMap};
use std::io::{self, Write};

type FResult = Result<(), String>;

enum Word {
    Native(fn(&mut Runtime) -> FResult),
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
    input: VecDeque<String>,
    dictionary: HashMap<&'a str, Word>,
    stack: Vec<i32>
}

impl <'a> Runtime<'a> {
    fn new() -> Runtime<'a> {
        Runtime {
            input: VecDeque::new(),
            dictionary: HashMap::new(),
            stack: vec![]
        }
    }

    fn append_input(&mut self, source: &str) {
        for name in source.split_whitespace() {
            self.input.push_back(name.to_string())
        }
    }

    fn eval(&mut self, source: &str) -> FResult {
        self.append_input(source);

        while let Some(name) = self.input.pop_front() {
            let result = self.eval_name(&name);

            if let Err(_) = result {
                return result;
            }
        }

        Ok(())
    }

    fn eval_name(&mut self, name: &str) -> FResult {
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

    fn eval_as_number(&mut self, name: &str) -> FResult {
        match name.parse() {
            Ok(num) => {
                self.stack.push(num);
                Ok(())
            },
            Err(_) => Err("Undefined word".to_string())
            // TODO: more descriptive error type
        }
    }

    fn eval_value(&mut self, value: &Word) -> FResult {
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

    fn push(&mut self, value: i32) {
        self.stack.push(value)
    }
}

fn rt_forth_print(forth: &mut Runtime) -> FResult {
    let value = try!(forth.pop().ok_or("Stack underflow".to_string()));
    println!("{}", value);
    Ok(())
}

fn rt_forth_add(forth: &mut Runtime) -> FResult {
    // TODO: these bits may be too clever - hard to read for
    // those unfamiliar with Rust
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| b + a))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_sub(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| b - a))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_mul(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| b * a))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_div(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| b / a))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn main() {
    let mut forth = Runtime::new();

    // TODO: put all this into Runtime::new, or such
    // as initializing the standard library
    forth.register(".", Word::Native(rt_forth_print));

    forth.register("+", Word::Native(rt_forth_add));
    forth.register("-", Word::Native(rt_forth_sub));

    forth.register("*", Word::Native(rt_forth_mul));
    forth.register("/", Word::Native(rt_forth_div));

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
