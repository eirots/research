use std::{collections::HashMap, fmt};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Lam { x: String, body: Expression },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Lam { x, body } => write!(f, "λ{}.{}", x, body),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Kont {
    Mt,
    Ar(Expression, Env, Box<Kont>),
    Fn(Value, Env, Box<Kont>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Addr(pub usize);

impl fmt::Display for Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "a{}", self.0)
    }
}

pub type Env = std::collections::HashMap<String, Addr>;

pub struct Store {
    next: usize,
    cells: HashMap<Addr, (Value, Env)>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            next: 0,
            cells: HashMap::new(),
        }
    }
    pub fn fresh(&mut self) -> Addr {
        let a = Addr(self.next);
        self.next += 1;
        a
    }

    pub fn insert(&mut self, a: Addr, v: (Value, Env)) {
        self.cells.insert(a, v);
    }

    pub fn get(&self, a: Addr) -> &(Value, Env) {
        self.cells
            .get(&a)
            .expect("Address wasn't found in store, that's not good")
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn lines(&self) -> Vec<String> {
        // stable ordering for UI
        let mut items: Vec<_> = self.cells.iter().collect();
        items.sort_by_key(|(a, _)| a.0);

        items
            .into_iter()
            .map(|(a, (v, rho))| format!("{a} ↦ ({v}, ρ size={})", rho.len()))
            .collect()
    }
}

pub struct State {
    pub control: Control, // either an Expression e or a Value v
    pub env: Env,         // ρ
    pub store: Store,     // σ
    pub kont: Kont,       // κ
}

impl State {
    pub fn env_lines(&self) -> Vec<String> {
        let mut items: Vec<_> = self.env.iter().collect();
        items.sort_by_key(|(k, _)| *k);

        items
            .into_iter()
            .map(|(x, a)| format!("{x} ↦ {a}"))
            .collect()
    }

    pub fn kont_lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut k = &self.kont;

        loop {
            match k {
                Kont::Mt => {
                    out.push("mt".to_string());
                    break;
                }
                Kont::Ar(e, rho, next) => {
                    out.push(format!("ar(•, {}, ρ size={})", e, rho.len()));
                    k = next.as_ref();
                }
                Kont::Fn(v, rho, next) => {
                    out.push(format!("fn({}, •, ρ size={})", v, rho.len()));
                    k = next.as_ref();
                }
            }
        }

        out
    }
}

pub enum Control {
    E(Expression),
    V(Value),
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Control::E(e) => write!(f, "{e}"),
            Control::V(v) => write!(f, "{v}"),
        }
    }
}

pub struct CESKMachine {
    state: State,
    finished: bool,
}

impl CESKMachine {
    pub fn new(initial_expression: Expression) -> Self {
        Self {
            state: State {
                control: Control::E(initial_expression),
                env: Env::new(),
                store: Store::new(),
                kont: Kont::Mt,
            },
            finished: false,
        }
    }

    pub fn step(&mut self) {
        if self.finished {
            return;
        }
        // move out of self.state without requiring Clone
        let state = std::mem::replace(
            &mut self.state,
            State {
                control: Control::E(Expression::LiteralInteger(0)),
                env: Env::new(),
                store: Store::new(),
                kont: Kont::Mt,
            },
        );

        self.state = CESKMachine::step_state(state);

        self.finished =
            matches!(self.state.control, Control::V(_)) && matches!(self.state.kont, Kont::Mt);
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn step_state(state: State) -> State {
        use Control::{E, V};
        use Expression as Ex;
        use Kont::{Ar, Fn, Mt};
        use Value as Val;

        let State {
            control,
            env,
            mut store,
            kont,
        } = state;

        match (control, env, kont) {
            // ⟨x, ρ, σ, κ⟩ → ⟨v, ρ′, σ, κ⟩ where σ(ρ(x)) = (v, ρ′)
            (E(Ex::Variable(x)), rho, k) => {
                let a = *rho
                    .get(&x)
                    .unwrap_or_else(|| panic!("unbound variable: {x}"));
                let (v, rho_prime) = store.get(a).clone();
                State {
                    control: V(v),
                    env: rho_prime,
                    store,
                    kont: k,
                }
            }

            // ⟨(e0 e1), ρ, σ, κ⟩ → ⟨e0, ρ, σ, ar(e1, ρ, κ)⟩
            (
                E(Ex::Application {
                    function_expression,
                    argument_expression,
                }),
                rho,
                k,
            ) => {
                let e0 = *function_expression;
                let e1 = *argument_expression;
                let k2 = Ar(e1, rho.clone(), Box::new(k));
                State {
                    control: E(e0),
                    env: rho,
                    store,
                    kont: k2,
                }
            }

            // Lambdas + integers are values (in the paper they are "v")
            (E(Ex::LiteralInteger(n)), rho, k) => State {
                control: V(Val::Int(n)),
                env: rho,
                store,
                kont: k,
            },

            (
                E(Ex::Lambda {
                    parameter_name,
                    body_expression,
                }),
                rho,
                k,
            ) => State {
                control: V(Val::Lam {
                    x: parameter_name,
                    body: *body_expression,
                }),
                env: rho,
                store,
                kont: k,
            },

            // ⟨v, ρ, σ, ar(e, ρ′, κ)⟩ → ⟨e, ρ′, σ, fn(v, ρ, κ)⟩
            (V(v), rho, Ar(e, rho_prime, k)) => {
                let k2 = Fn(v, rho, k);
                State {
                    control: E(e),
                    env: rho_prime,
                    store,
                    kont: k2,
                }
            }

            // ⟨v, ρ, σ, fn((λx.e), ρ′, κ)⟩ → ⟨e, ρ′[x↦a], σ[a↦(v,ρ)], κ⟩
            (V(varg), rho_arg, Fn(Val::Lam { x, body }, rho_fun, k)) => {
                let a = store.fresh();
                store.insert(a, (varg, rho_arg));
                let mut rho_fun2 = rho_fun;
                rho_fun2.insert(x, a);
                State {
                    control: E(body),
                    env: rho_fun2,
                    store,
                    kont: *k,
                }
            }

            // Done case: ⟨v, ρ, σ, mt⟩ is final. Here: just keep it as-is.
            (V(v), rho, Mt) => State {
                control: V(v),
                env: rho,
                store,
                kont: Mt,
            },

            // error branch, ideally never make it to this. Need it to avoid nonexhaustive matches
            (Control::V(_arg), _rho_arg, Kont::Fn(non_fun, _rho_fun, _k)) => {
                panic!("Attempted to apply non-function value: {:?}", non_fun);
            }
        }
    }
}

pub fn evaluate_expression(initial_expression: Expression) -> Value {
    let mut m = CESKMachine::new(initial_expression);
    while !m.is_finished() {
        m.step();
    }
    match &m.state.control {
        Control::V(v) => v.clone(),
        _ => unreachable!("finished machine must be in V(v) with Mt"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_int() {
        // ((λx.x) 42) => 42
        let expr = Expression::Application {
            function_expression: Box::new(Expression::Lambda {
                parameter_name: "x".to_string(),
                body_expression: Box::new(Expression::Variable("x".to_string())),
            }),
            argument_expression: Box::new(Expression::LiteralInteger(42)),
        };

        let result = evaluate_expression(expr);
        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_constant_function_applied_twice() {
        // (((λx.λy.x) 5) 6) => 5
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

        let result = evaluate_expression(expr);
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    fn test_bare_literal() {
        let expr = Expression::LiteralInteger(25);
        let result = evaluate_expression(expr);
        assert_eq!(result, Value::Int(25));
    }

    #[test]
    fn test_lexical_scoping_closure_capture() {
        // (((λx. (λy. x)) 5) 999) => 5, 5 should be captured
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
            argument_expression: Box::new(Expression::LiteralInteger(999)),
        };

        let result = evaluate_expression(expr);
        assert_eq!(result, Value::Int(5));
    }

    #[test]
    #[should_panic(expected = "Attempted to apply non-function value")]
    fn test_apply_non_function_panics() {
        // (1 2) should panic: trying to apply an int
        let expr = Expression::Application {
            function_expression: Box::new(Expression::LiteralInteger(1)),
            argument_expression: Box::new(Expression::LiteralInteger(2)),
        };

        let _ = evaluate_expression(expr);
    }
}
