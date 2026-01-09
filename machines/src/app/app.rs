use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::cesk_machine::cesk::{
    CESKMachine, Control as CesKControl, Expression as CesKExpr, Value as CesKVal,
};
use crate::{ck_machine::ck::CKMachine, ui::UiMode};

pub enum MachineKind {
    CK(CKMachine),
    CESK(CESKMachine),
}

pub struct App {
    pub mode: UiMode,
    pub machine: MachineKind,
    pub exit: bool,
    pub step: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            mode: UiMode::MainMenu,
            machine: MachineKind::CK(CKMachine::new(
                crate::ck_machine::ck::Expression::Application {
                    function_expression: Box::new(crate::ck_machine::ck::Expression::Lambda {
                        parameter_name: "x".to_string(),
                        body_expression: Box::new(crate::ck_machine::ck::Expression::Variable(
                            "x".to_string(),
                        )),
                    }),

                    argument_expression: Box::new(
                        crate::ck_machine::ck::Expression::LiteralInteger(42),
                    ),
                },
            )),
            exit: false,
            step: 0,
        }
    }
}

impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        let ch = match key.code {
            KeyCode::Char(c) => Some(c.to_ascii_lowercase()), //normalize UI inputs
            _ => None,
        };

        if ch == Some('q') {
            self.exit = true;
            return;
        }

        match &mut self.mode {
            UiMode::MainMenu => match ch {
                Some('1') => {
                    self.load_example_1();
                    self.mode = UiMode::RunningMachine;
                }
                Some('2') => {
                    self.load_example_2();
                    self.mode = UiMode::RunningMachine;
                }
                Some('3') => {
                    self.load_example_3();
                    self.mode = UiMode::RunningMachine;
                }
                Some('e') => {
                    self.mode = UiMode::EnterExpr {
                        buffer: String::new(),
                        error: None,
                    };
                }
                _ => {}
            },

            UiMode::RunningMachine => {
                if ch == Some('m') {
                    self.switch_machine();
                    return;
                }
                match key.code {
                    KeyCode::Right => self.step_forward(),
                    KeyCode::Esc => self.mode = UiMode::MainMenu,
                    _ => {}
                }
            }

            UiMode::EnterExpr { buffer, error } => match key.code {
                KeyCode::Esc => self.mode = UiMode::MainMenu,

                KeyCode::Backspace => {
                    buffer.pop();
                }

                KeyCode::Char(c) => {
                    buffer.push(c);
                }

                KeyCode::Enter => {
                    // TODO: parse `buffer` into an Expression
                    // On success:
                    // self.machine = MachineKind::CK(CKMachine::new(expr));
                    // self.step = 0;
                    // self.mode = UiMode::RunningMachine;
                    //
                    // On failure:
                    *error = Some("Expression parsing not wired up yet.".to_string());
                }

                _ => {}
            },
        }
    }

    fn machine_finished(&self) -> bool {
        match &self.machine {
            MachineKind::CK(m) => m.is_finished(),
            MachineKind::CESK(m) => m.is_finished(),
        }
    }
    fn load_example_1(&mut self) {
        self.step = 0;
        self.machine = MachineKind::CK(CKMachine::new(
            crate::ck_machine::ck::Expression::Application {
                function_expression: Box::new(crate::ck_machine::ck::Expression::Lambda {
                    parameter_name: "x".to_string(),
                    body_expression: Box::new(crate::ck_machine::ck::Expression::Variable(
                        "x".to_string(),
                    )),
                }),
                argument_expression: Box::new(crate::ck_machine::ck::Expression::LiteralInteger(
                    42,
                )),
            },
        ));
    }

    fn load_example_2(&mut self) {
        self.step = 0;
        self.machine = MachineKind::CK(CKMachine::new(
            crate::ck_machine::ck::Expression::Application {
                // ((λx. (λy. x)) 42) 99
                function_expression: Box::new(crate::ck_machine::ck::Expression::Application {
                    function_expression: Box::new(crate::ck_machine::ck::Expression::Lambda {
                        parameter_name: "x".to_string(),
                        body_expression: Box::new(crate::ck_machine::ck::Expression::Lambda {
                            parameter_name: "y".to_string(),
                            body_expression: Box::new(crate::ck_machine::ck::Expression::Variable(
                                "x".to_string(),
                            )),
                        }),
                    }),
                    argument_expression: Box::new(
                        crate::ck_machine::ck::Expression::LiteralInteger(42),
                    ),
                }),
                argument_expression: Box::new(crate::ck_machine::ck::Expression::LiteralInteger(
                    99,
                )),
            },
        ));
    }

    fn load_example_3(&mut self) {
        use crate::cesk_machine::cesk::Expression as E;

        self.step = 0;

        // ((((λa.(λx.((λy.λu.λv.a) 99)) 10) 123) 456) 789) => 10
        let expr = E::Application {
            function_expression: Box::new(E::Application {
                function_expression: Box::new(E::Application {
                    function_expression: Box::new(E::Application {
                        function_expression: Box::new(E::Lambda {
                            parameter_name: "a".to_string(),
                            body_expression: Box::new(E::Lambda {
                                parameter_name: "x".to_string(),
                                body_expression: Box::new(E::Application {
                                    // ((λy. λu. λv. a) 99)
                                    function_expression: Box::new(E::Lambda {
                                        parameter_name: "y".to_string(),
                                        body_expression: Box::new(E::Lambda {
                                            parameter_name: "u".to_string(),
                                            body_expression: Box::new(E::Lambda {
                                                parameter_name: "v".to_string(),
                                                body_expression: Box::new(E::Variable(
                                                    "a".to_string(),
                                                )),
                                            }),
                                        }),
                                    }),
                                    argument_expression: Box::new(E::LiteralInteger(99)),
                                }),
                            }),
                        }),
                        argument_expression: Box::new(E::LiteralInteger(10)),
                    }),
                    argument_expression: Box::new(E::LiteralInteger(123)),
                }),
                argument_expression: Box::new(E::LiteralInteger(456)),
            }),
            argument_expression: Box::new(E::LiteralInteger(789)),
        };

        self.machine = MachineKind::CESK(CESKMachine::new(expr));
    }

    fn step_forward(&mut self) {
        if self.machine_finished() {
            return; //lock moving forward if its finished
        }
        self.step += 1;
        match &mut self.machine {
            MachineKind::CK(m) => m.step(),
            MachineKind::CESK(m) => m.step(),
        }
    }

    fn switch_machine(&mut self) {
        // Move current machine out so we can replace it.
        // Use a dummy placeholder.
        let dummy = MachineKind::CK(CKMachine::new(
            crate::ck_machine::ck::Expression::LiteralInteger(0),
        ));
        let old = std::mem::replace(&mut self.machine, dummy);

        self.step = 0;

        self.machine = match old {
            MachineKind::CK(m) => {
                let expr_ck = m.state().current_expression.clone();
                let expr_cesk = ck_to_cesk_expr(expr_ck);
                MachineKind::CESK(CESKMachine::new(expr_cesk))
            }
            MachineKind::CESK(m) => {
                let expr_cesk = cesk_control_to_expr(&m.state().control);
                let expr_ck = cesk_to_ck_expr(expr_cesk);
                MachineKind::CK(CKMachine::new(expr_ck))
            }
        };
    }
}

fn ck_to_cesk_expr(e: crate::ck_machine::ck::Expression) -> CesKExpr {
    use crate::ck_machine::ck::Expression as CK;
    match e {
        CK::Variable(x) => CesKExpr::Variable(x),
        CK::LiteralInteger(n) => CesKExpr::LiteralInteger(n),
        CK::Lambda {
            parameter_name,
            body_expression,
        } => CesKExpr::Lambda {
            parameter_name,
            body_expression: Box::new(ck_to_cesk_expr(*body_expression)),
        },
        CK::Application {
            function_expression,
            argument_expression,
        } => CesKExpr::Application {
            function_expression: Box::new(ck_to_cesk_expr(*function_expression)),
            argument_expression: Box::new(ck_to_cesk_expr(*argument_expression)),
        },
    }
}

fn cesk_to_ck_expr(e: CesKExpr) -> crate::ck_machine::ck::Expression {
    use crate::ck_machine::ck::Expression as CK;
    match e {
        CesKExpr::Variable(x) => CK::Variable(x),
        CesKExpr::LiteralInteger(n) => CK::LiteralInteger(n),
        CesKExpr::Lambda {
            parameter_name,
            body_expression,
        } => CK::Lambda {
            parameter_name,
            body_expression: Box::new(cesk_to_ck_expr(*body_expression)),
        },
        CesKExpr::Application {
            function_expression,
            argument_expression,
        } => CK::Application {
            function_expression: Box::new(cesk_to_ck_expr(*function_expression)),
            argument_expression: Box::new(cesk_to_ck_expr(*argument_expression)),
        },
    }
}

// When switching away from CESK, we need *some* expression.
// If we're in a value, reify it to syntax (int or lambda).
fn cesk_control_to_expr(control: &CesKControl) -> CesKExpr {
    match control {
        CesKControl::E(e) => e.clone(),
        CesKControl::V(v) => match v {
            CesKVal::Int(n) => CesKExpr::LiteralInteger(*n),
            CesKVal::Lam { x, body } => CesKExpr::Lambda {
                parameter_name: x.clone(),
                body_expression: Box::new(body.clone()),
            },
        },
    }
}
