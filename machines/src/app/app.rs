use ratatui::crossterm::event::{KeyCode, KeyEvent};

//use crate::cek_machine::CEKMachine;
use crate::ck_machine::ck::CKMachine;

pub enum MachineKind {
    CK(CKMachine),
    //CEK(CEKMachine),
}

pub struct App {
    pub machine: MachineKind,
    pub exit: bool,
    pub step: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            machine: MachineKind::CK(CKMachine::new(
                crate::ck_machine::ck::Expression::LiteralInteger(5),
            )),
            exit: false,
            step: 0,
        }
    }
}

impl App {
    pub fn handle_key_event(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Right => {
                self.step_forward();
            }
            KeyCode::Left => {
                self.step_back();
            }
            //Char('m') => {
            //    self.toggle_machine();
            //}
            _ => {}
        }
    }

    fn step_forward(&mut self) {
        self.step += 1;
        match &mut self.machine {
            MachineKind::CK(m) => m.step(), // your CK step
                                            //MachineKind::CEK(m) => m.step(), // your CEK step
        }
    }

    fn step_back(&mut self) {
        if self.step > 0 {
            self.step -= 1;
            // up to you whether you support “backwards”
        }
    }

    // fn toggle_machine(&mut self) {
    //     self.machine = match std::mem::take(&mut self.machine) {
    //         MachineKind::CK(_) => MachineKind::CEK(CEKMachine::new()),
    //         MachineKind::CEK(_) => MachineKind::CK(CKMachine::new()),
    //     };
    //     self.step = 0;
    // }
}
