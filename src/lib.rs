use crate::vm::interpret;

pub mod compiler;
pub mod chunk;
pub mod debug;
pub mod value;
pub mod scanner;
pub mod vm;
pub mod string_intern;

fn repl() {
    loop {
        print!("> ");

        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();

        interpret(&line);
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_concat_string_intern() {
        interpret(r#"
            "a" + "a" + "aa"
        "#);
    }

    #[test]
    fn test_concat_twice() {
        interpret(r#"
            "st" + "ri" + "ng"
        "#);
    }

    #[test]
    fn string_concat() {
        interpret(r#" "a" + "b" "#);
    }

    #[test]
    fn num_add() {
        interpret(r#" 1 + 2 "#);
    }

    #[test]
    fn num_div() {
        interpret(r#" 10 / 2 "#);
    }

    #[test]
    fn test_expression() {
        interpret(r#"
            !(5 - 4 > 3 * 2 == !nil)
        "#);
    }
}