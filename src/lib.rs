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
    fn recursion_and_natives() {
        interpret(
            r#"
            fun fib(n) {
                if (n < 2) return n;
                return fib(n - 2) + fib(n - 1);
            }
            
            var start = clock();
            print fib(26);
            print clock() - start;
        "#,
        );
    }

    #[test]
    fn natives() {
        interpret(
            r#"
            print clock();
        "#,
        );
    }

    #[test]
    fn higher_order_fuction() {
        interpret(
            r#"
            fun foo(text) {
                return text + text;
            }

            fun call(fn, arg) {
                return fn(arg);
            }

            print call(foo, "blah");
        "#,
        );
    }

    #[test]
    fn function_return() {
        interpret(
            r#"
            fun foo() {
                return "blah";
            }

            print foo();
        "#,
        );
    }

    #[test]
    fn nested_function_calls() {
        interpret(
            r#"
            fun bar(a) {
                print a;
            }

            fun foo(b) {
                bar(b);
            }

            foo("blah");
        "#,
        );
    }

    #[test]
    fn call_function_with_args() {
        interpret(
            r#"
            fun doSomething(text) {
                print text;
            }

            doSomething("blah");
        "#,
        );
    }

    #[test]
    fn call_function() {
        interpret(
            r#"
            fun doSomething() {
                print "blah";
            }

            doSomething();
        "#,
        );
    }

    #[test]
    fn print_function_name() {
        interpret(
            r#"
            fun doSomething() {
                print "blah";
            }

            print doSomething;
        "#,
        );
    }

    #[test]
    fn for_loop() {
        interpret(
            r#"
            for (var x = 50; x < 51; x = x + 1) {
                print x;
            }
        "#,
        );
    }

    #[test]
    fn while_loop() {
        interpret(
            r#"
            var x = 1;
            while (x < 2) {
                print x;
                x = x + 1;
            }
        "#,
        );
    }

    #[test]
    fn if_condition() {
        interpret(
            r#"
            var x = 1;
            if (x < 5) {
                print x;
                x = x + 1;
            }
        "#,
        );
    }

    #[test]
    fn or() {
        interpret(
            r#"
            if (true or true) {
                print "a";
            }
            if (true or false) {
                print "b";
            }
            if (false or true) {
                print "c";
            }
            if (false or false) {
                print "d";
            }
        "#,
        );
    }

    #[test]
    fn and() {
        interpret(
            r#"
            if (true and true) {
                print "a";
            }
            if (true and false) {
                print "b";
            }
            if (false and true) {
                print "c";
            }
            if (false and false) {
                print "d";
            }
        "#,
        );
    }

    #[test]
    fn if_else() {
        interpret(
            r#"
            if (false) {
                print "a";
            } else {
                print "b";
            }
        "#,
        );
    }

    #[test]
    fn if_then() {
        interpret(
            r#"
            if (true) {
                print "a";
            }

            if (false) {
                print "b";
            }
        "#,
        );
    }

    #[test]
    fn begin_end_scope_with_override() {
        interpret(
            r#"
            var a = 99;
            {
                 a = 50;
            }
            print a;
        "#,
        );
    }

    #[test]
    fn begin_end_scope_with_same_var() {
        interpret(
            r#"
            var a = 99;
            {
                var a = 50;
            }
            print a;
        "#,
        );
    }

    #[test]
    fn begin_end_scope_with_var() {
        interpret(
            r#"
            var a = 99;
            {
                var b = 50;
            }
            print a;
        "#,
        );
    }

    #[test]
    fn begin_end_scope() {
        interpret(
            r#"
            var a = 99;
            {
            }
            print a;
        "#,
        );
    }

    #[test]
    fn scopes_and_locals() {
        interpret(
            r#"
            var a = 1;
            {
                var a = 2;
                {
                    var a = 3;
                    print a;
                }
                print a;
            }
            print a;
        "#,
        );
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
    fn hello_world() {
        interpret(
            r#"
            print "hello world";
        "#,
        );
    }
}
