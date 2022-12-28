pub mod compiler;
pub mod chunk;
pub mod debug;
pub mod value;
pub mod scanner;
pub mod vm;
pub mod string_intern;

#[cfg(test)]
mod tests {
    use crate::vm::interpret;

    #[test]
    fn test_globals() {
        interpret(r#"
            var beverage = "cafe au lait";
            var breakfast = "beignets with " + beverage;
            print breakfast;
        "#);
    }

    #[test]
    fn test_concat_string_intern() {
        interpret(r#"
            print "a" + "a" + "aa";
        "#);
    }

    #[test]
    fn test_concat_twice() {
        interpret(r#"
            print "st" + "ri" + "ng";
        "#);
    }

    #[test]
    fn string_concat() {
        interpret(r#"
            print "a" + "b";
        "#);
    }

    #[test]
    fn num_add() {
        interpret(r#"
            print 1 + 2;
        "#);
    }

    #[test]
    fn num_div() {
        interpret(r#"
            print 10 / 2;
        "#);
    }

    #[test]
    fn test_expression() {
        interpret(r#"
            print !(5 - 4 > 3 * 2 == !nil);
        "#);
    }
}