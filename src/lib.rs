mod stdlib;

use std::collections::{VecDeque, HashMap};

pub type FResult = Result<(), String>;

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

pub struct Runtime {
    input: VecDeque<String>,
    dictionary: HashMap<String, Word>,
    stack: Vec<i32>
}

impl Runtime {
    pub fn new() -> Runtime {
        let mut forth = Runtime {
            input: VecDeque::new(),
            dictionary: HashMap::new(),
            stack: vec![]
        };
        
        stdlib::register_stdlib(&mut forth);

        forth
    }

    fn append_input(&mut self, source: &str) {
        for name in source.split_whitespace() {
            self.input.push_back(name.to_string())
        }
    }

    fn prepend_names(&mut self, names: &Vec<String>) {
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

    fn resolve(&self, name: &str) -> Result<Word, String> {
        match self.dictionary.get(&name.to_lowercase()) {
            Some(w) => Ok(w.clone()),
            None => {
                name.parse()
                    .map(|num| Word::Number(num))
                    .or(Err(format!("Undefined word \"{}\"", name)))

            }
        }
    }

    pub fn eval(&mut self, source: &str) -> FResult {
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
        self.resolve(name).and_then(|word| self.eval_word(&word))
    }

    fn eval_word(&mut self, word: &Word) -> FResult {
        match word {
            &Word::Native(callback) => callback(self),
            &Word::Colon(ref definition) => {
                self.prepend_names(definition);
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

    fn pop(&mut self) -> Option<i32> {
        self.stack.pop()
    }

    fn push(&mut self, value: i32) {
        self.stack.push(value)
    }
}

