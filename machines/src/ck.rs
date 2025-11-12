use std::fmt;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct MachineState {
    pub current_expression: Expression,
    pub continuation_stack: Vec<ContinuationFrame>,
}

pub enum MachineStep {
    Continue(MachineState),
    Done(Expression),
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
    let mut state = MachineState {
        current_expression: initial_expression,
        continuation_stack: Vec::new(),
    };

    loop {
        match step_machine_state(state) {
            MachineStep::Continue(next_state) => {
                state = next_state;
            }
            MachineStep::Done(final_value) => {
                return final_value;
            }
        }
    }
}
