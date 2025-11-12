mod ck;
use ck::Expression;
use ck::evaluate_expression;
/*
testing:
    ((λx.x) 42) -> 42
    (Identity function on an integer)
*/

fn test_ck_1() {
    let expr = Expression::Application {
        function_expression: Box::new(Expression::Lambda {
            parameter_name: "x".to_string(),
            body_expression: Box::new(Expression::Variable("x".to_string())),
        }),

        argument_expression: Box::new(Expression::LiteralInteger(42)),
    };

    println!("Testing the identity function applied to an integer. \n\t{expr}");

    let result = evaluate_expression(expr);

    println!("\t{result:?}\n");
}

/*
testing constant function applied twice
    (((λx.λy.x)5) 6) -> 5
*/
fn test_ck_2() {
    let expr = Expression::Application {
        function_expression: Box::new(Expression::Application {
            function_expression: Box::new(Expression::Lambda {
                parameter_name: "x".to_string(),
                body_expression: Box::new(Expression::Lambda {
                    parameter_name: "y".to_string(),
                    body_expression: Box::new(Expression::Variable("x".to_string())),
                }),
            }),
            argument_expression: Box::new(Expression::LiteralInteger(5)),
        }),
        argument_expression: Box::new(Expression::LiteralInteger(6)),
    };
    println!("Testing constant function applied twice. \n\t{expr}");

    let result = evaluate_expression(expr);
    println!("\t{result:?}\n");
}

/*
testing a bare literal
*/
fn test_ck_3() {
    let expr = Expression::LiteralInteger(25);
    println!("Testing a bare literal integer. \n\t{expr}");
    let result = evaluate_expression(expr);
    println!("\t{result:?}\n");
}

fn run_ck_tests() {
    println!("Beginning tests");

    test_ck_1();
    test_ck_2();
    test_ck_3();

    println!("Ending tests");
}

fn main() {
    run_ck_tests();
}
