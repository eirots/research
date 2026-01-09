use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]

pub enum Expression {
    /*
       <expr> ::= <variable>
               | "\"
                  <identifier>
                  "."
                  <expr>
                | "(" <expr> <expr> ")"
                | <integer>
    */
    Variable(String),
    Lambda {
        parameter_name: String,
        body_expression: Box<Expression>,
    },
    Application {
        function_expression: Box<Expression>,
        argument_expression: Box<Expression>,
    },
    LiteralInteger(i64),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Variable(name) => write!(f, "{name}"),

            Expression::LiteralInteger(n) => write!(f, "{n}"),

            Expression::Lambda {
                parameter_name,
                body_expression,
            } => {
                write!(f, "λ{}.{}", parameter_name, body_expression)
            }

            Expression::Application {
                function_expression,
                argument_expression,
            } => {
                write!(f, "({} {})", function_expression, argument_expression)
            }
        }
    }
}

//continuations as a stack. Evaluate the head, push frames.
#[derive(Clone, Debug)]
pub enum ContinuationFrame {
    EvaluateFunctionThenApplyArgument(Expression), //evaluating function part, need to remember argument for later
    EvaluateArgumentThenApplyFunction(Expression), //function has been evaluated, now evaluating the argument.
}

impl fmt::Display for ContinuationFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContinuationFrame::EvaluateFunctionThenApplyArgument(arg) => {
                write!(f, "⟨• {}⟩   (waiting for function)", arg)
            }
            ContinuationFrame::EvaluateArgumentThenApplyFunction(fun) => {
                write!(f, "⟨• {}⟩   (waiting for argument)", fun)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MachineState {
    pub current_expression: Expression,
    pub continuation_stack: Vec<ContinuationFrame>,
}
impl MachineState {
    pub fn stack_lines(&self) -> Vec<String> {
        if self.continuation_stack.is_empty() {
            return vec!["∅".to_string()];
        }
        self.continuation_stack
            .iter()
            .rev()
            .map(|frame| frame.to_string())
            .collect()
    }
}
impl Default for MachineState {
    fn default() -> Self {
        Self {
            current_expression: Expression::LiteralInteger(0),
            continuation_stack: Vec::new(),
        }
    }
}
pub enum MachineStep {
    Continue(MachineState),
    Done(Expression),
}

pub struct CKMachine {
    state: MachineState,
    finished: bool,
}

impl CKMachine {
    pub fn new(initial_expression: Expression) -> Self {
        Self {
            state: MachineState {
                current_expression: initial_expression,
                continuation_stack: Vec::new(),
            },
            finished: false,
        }
    }

    pub fn step(&mut self) {
        if self.finished {
            return;
        }
        let state = std::mem::take(&mut self.state);

        match step_machine_state(state) {
            MachineStep::Continue(next_state) => {
                self.state = next_state;
            }

            MachineStep::Done(final_value) => {
                self.state.current_expression = final_value;
                self.state.continuation_stack.clear();
                self.finished = true;
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn state(&self) -> &MachineState {
        &self.state
    }
}

fn substitute_variable_with_expression(
    target_expression: Expression,
    variable_name: &str,
    replacement_expression: &Expression,
) -> Expression {
    match target_expression {
        Expression::Variable(ref name) if name == variable_name => replacement_expression.clone(),

        Expression::Variable(_) => target_expression,

        Expression::Lambda {
            parameter_name,
            body_expression,
        } => {
            if parameter_name == variable_name {
                //shadowed variable, no sub
                Expression::Lambda {
                    parameter_name,
                    body_expression,
                }
            } else {
                Expression::Lambda {
                    parameter_name,
                    body_expression: Box::new(substitute_variable_with_expression(
                        *body_expression,
                        variable_name,
                        replacement_expression,
                    )),
                }
            }
        }
        Expression::Application {
            function_expression,
            argument_expression,
        } => Expression::Application {
            function_expression: Box::new(substitute_variable_with_expression(
                *function_expression,
                variable_name,
                replacement_expression,
            )),
            argument_expression: Box::new(substitute_variable_with_expression(
                *argument_expression,
                variable_name,
                replacement_expression,
            )),
        },
        Expression::LiteralInteger(_) => target_expression,
    }
}

fn step_machine_state(state: MachineState) -> MachineStep {
    let MachineState {
        current_expression,
        mut continuation_stack,
    } = state;

    match current_expression {
        Expression::Application {
            function_expression,
            argument_expression,
        } => {
            continuation_stack.push(ContinuationFrame::EvaluateFunctionThenApplyArgument(
                *argument_expression,
            ));
            MachineStep::Continue(MachineState {
                current_expression: *function_expression,
                continuation_stack,
            })
        }

        Expression::Lambda { .. } | Expression::LiteralInteger(_) => {
            match continuation_stack.pop() {
                None => {
                    // finished, return the current value
                    MachineStep::Done(current_expression)
                }

                Some(ContinuationFrame::EvaluateFunctionThenApplyArgument(argument_expression)) => {
                    continuation_stack.push(ContinuationFrame::EvaluateArgumentThenApplyFunction(
                        current_expression,
                    ));
                    MachineStep::Continue(MachineState {
                        current_expression: argument_expression,
                        continuation_stack,
                    })
                }

                Some(ContinuationFrame::EvaluateArgumentThenApplyFunction(function_expression)) => {
                    match function_expression {
                        Expression::Lambda {
                            parameter_name,
                            body_expression,
                        } => {
                            let substituted = substitute_variable_with_expression(
                                *body_expression,
                                &parameter_name,
                                &current_expression,
                            );
                            MachineStep::Continue(MachineState {
                                current_expression: substituted,
                                continuation_stack,
                            })
                        }
                        _ => {
                            panic!(
                                "Attempted to apply non-lambda expression: {:?}",
                                function_expression
                            );
                        }
                    }
                }
            }
        }

        Expression::Variable(name) => {
            panic!("Unbound variable encountered at runtime: {}", name);
        }
    }
}

pub fn evaluate_expression(initial_expression: Expression) -> Expression {
    //let mut state = MachineState {
    //    current_expression: initial_expression,
    //    continuation_stack: Vec::new(),
    //};
    //
    //loop {
    //    match step_machine_state(state) {
    //        MachineStep::Continue(next_state) => {
    //            state = next_state;
    //        }
    //        MachineStep::Done(final_value) => {
    //            return final_value;
    //        }
    //    }
    //}

    let mut machine = CKMachine::new(initial_expression);
    while !machine.is_finished() {
        machine.step();
    }
    machine.state.current_expression.clone()
}

#[allow(dead_code, unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn test_ck_1() {
        let expr = Expression::Application {
            function_expression: Box::new(Expression::Lambda {
                parameter_name: "x".to_string(),
                body_expression: Box::new(Expression::Variable("x".to_string())),
            }),

            argument_expression: Box::new(Expression::LiteralInteger(42)),
        };

        //println!("Testing the identity function applied to an integer. \n\t{expr}");

        let result = evaluate_expression(expr); //println!("\t{result:?}\n");

        assert_eq!(result, Expression::LiteralInteger(42));
    }

    /*
    testing constant function applied twice
        (((λx.λy.x) 5) 6) -> 5
    */
    #[test]
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
        //println!("Testing constant function applied twice. \n\t{expr}");

        let result = evaluate_expression(expr);
        //println!("\t{result:?}\n");

        assert_eq!(result, Expression::LiteralInteger(5)); //LiteralInteger(5)
    }

    /*
    testing a bare literal
    */
    #[test]
    fn test_ck_3() {
        let expr = Expression::LiteralInteger(25);
        //println!("Testing a bare literal integer. \n\t{expr}");
        let result = evaluate_expression(expr);
        //println!("\t{result:?}\n");

        assert_eq!(result, Expression::LiteralInteger(25)); //LiteralInteger(25)
    }

    //pub fn run_ck_tests() {
    //    println!("Beginning tests");
    //
    //    test_ck_1();
    //    test_ck_2();
    //    test_ck_3();
    //
    //    println!("Ending tests");
    //}
}
