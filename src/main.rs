use std::collections::{VecDeque, HashMap};
use std::io::{self, Write};

type FResult = Result<(), String>;

enum Word {
    Native(fn(&mut Runtime) -> FResult),
    Colon(Vec<String>),
    Number(i32)
}

impl Clone for Word {
    fn clone(&self) -> Self {
        match self {
            &Word::Native(callback) => Word::Native(callback),
            &Word::Colon(ref definition) => Word::Colon(definition.clone()),
            &Word::Number(num) => Word::Number(num)
        }
    }
}

struct Runtime {
    input: VecDeque<String>,
    dictionary: HashMap<String, Word>,
    stack: Vec<i32>
}

impl Runtime {
    fn new() -> Runtime {
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

    // TODO: fix inconsistent parameter types
    fn prepend_input(&mut self, names: &Vec<String>) {
        // To execute a user-defined colon definition,
        // put it at the beginning of the input queue
        for name in names.iter().rev() {
            self.input.push_front(name.to_string())
        }
    }

    // TODO: perhaps move interaction with console here
    //   so that Forth can read new input if it needs
    //   for example, if in a colon definition and ; has not
    //   yet been reached
    fn parse(&mut self) -> Option<String> {
        self.input.pop_front()
    }

    fn resolve(&self, name: &str) -> Option<Word> {
        match self.dictionary.get(&name.to_lowercase()) {
            Some(w) => Some(w.clone()),
            None => name.parse().ok().map(|num| Word::Number(num))
        }
    }

    fn eval(&mut self, source: &str) -> FResult {
        self.append_input(source);

        while let Some(name) = self.parse() {
            let result = self.eval_name(&name);

            if let Err(_) = result {
                self.input.clear();
                return result;
            }
        }

        Ok(())
    }

    fn eval_name(&mut self, name: &str) -> FResult {
        if let Some(word) = self.resolve(name) {
            self.eval_word(&word)
        } else {
            Err(format!("Undefined word \"{}\"", name))
        }
    }

    fn eval_word(&mut self, word: &Word) -> FResult {
        match word {
            &Word::Native(callback) => callback(self),
            &Word::Colon(ref definition) => {
                self.prepend_input(definition);
                Ok(())
            },
            &Word::Number(num) => {
                self.push(num);
                Ok(())
            }
        }
    }

    fn register(&mut self, name: &str, word: Word) {
        self.dictionary.insert(name.to_lowercase(), word);
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

fn rt_forth_eq(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| (b == a) as i32))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_gt(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| (b > a) as i32))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_lt(forth: &mut Runtime) -> FResult {
    let success = forth.pop()
        .and_then(|a| forth.pop().map(|b| (b < a) as i32))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_invert(forth: &mut Runtime) -> FResult {
    let value = try!(forth.pop().ok_or("Stack underflow".to_string()));
    forth.push((value == 0) as i32);
    Ok(())
}

fn rt_forth_dup(forth: &mut Runtime) -> FResult {
    let value = try!(forth.pop().ok_or("Stack underflow".to_string()));
    forth.push(value);
    forth.push(value);
    Ok(())
}

fn rt_forth_drop(forth: &mut Runtime) -> FResult {
    try!(forth.pop().ok_or("Stack underflow".to_string()));
    Ok(())
}

fn rt_forth_colon(forth: &mut Runtime) -> FResult {
    // TODO: Maybe some sort of convenience method for this?
    let name = try!(forth.parse().ok_or("Attempt to use zero-length string as name".to_string()));

    // TODO: perhaps a colon def should be words rather than names
    let mut definition = vec![];

    // TODO: should handle "( n -- )"-like strings
    //       I think it's a comment or specification of parameters??
    while let Some(word) = forth.parse() {
        if word == ";" {
            break;
        }

        definition.push(word)
    }

    forth.register(&name, Word::Colon(definition));

    Ok(())
}

fn rt_forth_if(forth: &mut Runtime) -> FResult {
    // TODO: unary op helper?
    let condition = try!(forth.pop().ok_or("Stack underflow".to_string())) != 0;
    let mut words = vec![];

    let mut in_consequent = true;

    while let Some(word) = forth.parse() {
        if word.to_lowercase() == "then" {
            break;
        }

        if word.to_lowercase() == "else" {
            in_consequent = false;
        } else if (condition && in_consequent) || (!condition && !in_consequent) {
            words.push(word);
        }
    }

    forth.prepend_input(&words);

    Ok(())
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

    forth.register("=", Word::Native(rt_forth_eq));
    forth.register(">", Word::Native(rt_forth_gt));
    forth.register("<", Word::Native(rt_forth_lt));

    forth.register("invert", Word::Native(rt_forth_invert));

    forth.register("dup", Word::Native(rt_forth_dup));
    forth.register("drop", Word::Native(rt_forth_drop));

    forth.register(":", Word::Native(rt_forth_colon));

    forth.register("if", Word::Native(rt_forth_if));

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
