use std::collections::{VecDeque, HashMap};
use std::io::{self, Write};

type FResult = Result<(), String>;

enum Word {
    Native(fn(&mut Runtime) -> FResult),
    Colon(Vec<String>),
    Number(i32) // is this a thing, in Forth?
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

    // TODO: perhaps move interaction with console here
    //   so that Forth can read new input if it needs
    //   for example, if in a colon definition and ; has not
    //   yet been reached
    fn parse(&mut self) -> Option<String> {
        self.input.pop_front()
    }

    fn eval(&mut self, source: &str) -> FResult {
        self.append_input(source);

        while let Some(name) = self.parse() {
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
            // TODO: is it possible to clean this up? to not use clone?
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
            //   so that error message could say what word
            //   and highlight in code
        }
    }

    fn eval_value(&mut self, value: &Word) -> FResult {
        match value {
            &Word::Native(callback) => callback(self),
            &Word::Colon(ref definition) => {
                // To execute a user-defined colon definition,
                // put it at the beginning of the input queue
                for name in definition.iter().rev() {
                    self.input.push_front(name.to_string())
                }
                Ok(())
            },
            &Word::Number(num) => Err(format!("not actually sure what to do with this... {}", num)),
        }
    }

    fn register(&mut self, name: String, word: Word) {
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

fn rt_forth_dup(forth: &mut Runtime) -> FResult {
    let value = try!(forth.pop().ok_or("Stack underflow".to_string()));
    forth.push(value);
    forth.push(value);
    Ok(())
}

fn rt_forth_colon(forth: &mut Runtime) -> FResult {
    // TODO: Maybe some sort of convenience method for this?
    let name = try!(forth.parse().ok_or("Attempt to use zero-length string as name".to_string()));

    let mut definition = vec![];

    // TODO: should handle "( n -- )"-like strings
    //       I think it's a comment or specification of parameters??
    while let Some(word) = forth.parse() {
        if word == ";" {
            break;
        }

        definition.push(word)
    }

    forth.register(name, Word::Colon(definition));

    Ok(())

}

fn main() {
    let mut forth = Runtime::new();

    // TODO: put all this into Runtime::new, or such
    // as initializing the standard library
    forth.register(".".to_string(), Word::Native(rt_forth_print));

    forth.register("+".to_string(), Word::Native(rt_forth_add));
    forth.register("-".to_string(), Word::Native(rt_forth_sub));

    forth.register("*".to_string(), Word::Native(rt_forth_mul));
    forth.register("/".to_string(), Word::Native(rt_forth_div));

    forth.register("dup".to_string(), Word::Native(rt_forth_dup));

    forth.register(":".to_string(), Word::Native(rt_forth_colon));

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
