pub mod chunk;
pub mod compiler;
pub mod debug;
pub mod rc_slice;
pub mod scanner;
pub mod string_intern;
pub mod value;
pub mod vm;

#[cfg(test)]
mod tests {
    use crate::vm::interpret;

    #[test]
    fn if_else() {
        interpret(r#"
            if (false) {
                print "a";
            } else {
                print "b";
            }
        "#);
    }

    #[test]
    fn if_then() {
        interpret(r#"
            if (true) {
                print "a";
            }

            if (false) {
                print "b";
            }
        "#);
    }
    
    #[test]
    fn scopes_and_locals() {
        interpret(r#"
            var a = 1;
            {
                var b = 2;
                print b;
            }
            print a;
        "#);
    }

    #[test]
    fn global_assignment() {
        interpret(
            r#"
            var breakfast = "beignets";
            var beverage = "cafe au lait";
            breakfast = breakfast + " with " + beverage;
            
            print breakfast;
        "#,
        );
    }

    #[test]
    fn globals() {
        interpret(
            r#"
            var beverage = "cafe au lait";
            var breakfast = "beignets with " + beverage;
            print breakfast;
        "#,
        );
    }

    #[test]
    fn concat_string_intern() {
        interpret(
            r#"
            print "a" + "a" + "aa";
        "#,
        );
    }

    #[test]
    fn concat_twice() {
        interpret(
            r#"
            print "st" + "ri" + "ng";
        "#,
        );
    }

    #[test]
    fn string_concat() {
        interpret(
            r#"
            print "a" + "b";
        "#,
        );
    }

    #[test]
    fn num_add() {
        interpret(
            r#"
            print 1 + 2;
        "#,
        );
    }

    #[test]
    fn num_div() {
        interpret(
            r#"
            print 10 / 2;
        "#,
        );
    }

    #[test]
    fn test_expression() {
        interpret(
            r#"
            print !(5 - 4 > 3 * 2 == !nil);
        "#,
        );
    }
}
