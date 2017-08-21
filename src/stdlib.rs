use Runtime;
use FResult;
use Word;

fn rt_forth_print(forth: &mut Runtime) -> FResult {
    let value = try!(forth.pop().ok_or("Stack underflow".to_string()));
    println!("{}", value);
    Ok(())
}

fn unary_word<F>(forth: &mut Runtime, callback: F) -> FResult
where
    F: FnMut(i32) -> i32,
{
    forth
        .pop()
        .map(callback)
        .map(|result| forth.push(result))
        .ok_or("Stack underflow".to_string())
}


fn binary_word<F>(forth: &mut Runtime, mut callback: F) -> FResult
where
    F: FnMut(i32, i32) -> i32,
{
    let success = forth
        .pop()
        .and_then(|a| forth.pop().map(|b| callback(b, a)))
        .map(|result| forth.push(result));

    success.ok_or("Stack underflow".to_string())
}

fn rt_forth_add(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| a + b)
}

fn rt_forth_sub(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| a - b)
}

fn rt_forth_mul(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| a * b)
}

fn rt_forth_div(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| a / b)
}

fn rt_forth_eq(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| (a == b) as i32)
}

fn rt_forth_gt(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| (a > b) as i32)
}

fn rt_forth_lt(forth: &mut Runtime) -> FResult {
    binary_word(forth, |a, b| (a < b) as i32)
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
    let name = try!(forth.parse().ok_or(
        "Attempt to use zero-length string as name".to_string(),
    ));

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

    forth.prepend_names(&words);

    Ok(())
}

pub fn register_stdlib(forth: &mut Runtime) {
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
}
