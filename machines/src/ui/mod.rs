use ratatui::{
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};

use crate::app::App;
use crate::app::app::MachineKind;

pub enum UiMode {
    MainMenu,
    EnterExpr {
        buffer: String,
        error: Option<String>,
    },
    RunningMachine,
}

pub fn render_app(area: Rect, buf: &mut Buffer, app: &App) {
    match &app.mode {
        UiMode::MainMenu => render_main_menu(area, buf, app),
        UiMode::EnterExpr { buffer, error } => {
            render_expression_input(area, buf, buffer, error.as_deref())
        }
        UiMode::RunningMachine => AppView { app }.render(area, buf),
    }
}

struct AppView<'a> {
    app: &'a App,
}

impl<'a> Widget for AppView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let finished = match &self.app.machine {
            MachineKind::CK(m) => m.is_finished(),
            MachineKind::CESK(m) => m.is_finished(),
        };
        // Split whole screen: top = machine, bottom = controls
        let [top, bottom] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .areas(area);

        // Further split top horizontally: [ expr | stack ]
        let [expr_area, stack_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .areas(top);

        // title section
        let machine_title = match self.app.machine {
            MachineKind::CK(_) => " CK Machine ",
            MachineKind::CESK(_) => " CESK Machine ",
        };
        let title = Line::from(machine_title).bold().centered();

        // get current machine state (render-ready text)
        let (left_lines, stack_lines): (Vec<String>, Vec<String>) = match &self.app.machine {
            MachineKind::CK(machine) => {
                let state = machine.state();
                let left_lines = vec![
                    format!("Step: {}", self.app.step),
                    format!("Status: {}", if finished { "FINISHED" } else { "running" }),
                    state.current_expression.to_string(),
                ];
                let stack_lines = state.stack_lines();
                (left_lines, stack_lines)
            }

            MachineKind::CESK(machine) => {
                let state = machine.state();

                // NOTE: these helpers are from cesk.rs:
                // - state.env_lines()
                // - state.kont_lines()
                // - state.store.lines()
                // - state.store.len()

                let mut left_lines = vec![
                    format!("Step: {}", self.app.step),
                    format!("Status: {}", if finished { "FINISHED" } else { "running" }),
                    format!("Control: {}", state.control),
                    format!("ρ size: {}", state.env.len()),
                    format!("σ size: {}", state.store.len()),
                    "".to_string(),
                    "Env (ρ):".to_string(),
                ];
                left_lines.extend(state.env_lines());

                left_lines.push("".to_string());
                left_lines.push("Store (σ):".to_string());
                left_lines.extend(state.store.lines());

                let stack_lines = state.kont_lines();
                (left_lines, stack_lines)
            }
        };

        // Left: current expression
        let expr_block = Block::bordered()
            .title(title.clone())
            .border_set(border::THICK);
        let expr_body = Text::from(left_lines.into_iter().map(Line::from).collect::<Vec<_>>());

        Paragraph::new(expr_body)
            .block(expr_block)
            .render(expr_area, buf);

        let stack_title = Line::from(" Stack ").bold().centered();

        let stack_block = Block::bordered()
            .title(stack_title)
            .border_set(border::THICK);

        let stack_body = Text::from(stack_lines.into_iter().map(Line::from).collect::<Vec<_>>());
        Paragraph::new(stack_body)
            .block(stack_block)
            .render(stack_area, buf);

        // ---- Bottom: controls ----
        let controls = Line::from(vec![
            " Step back ".into(),
            "<Left>".blue().bold(),
            " Step forward ".into(),
            "<Right>".blue().bold(),
            " Switch machine ".into(),
            "<M>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ]);

        let bottom_block = Block::bordered()
            .title_bottom(controls.centered())
            .border_set(border::THICK);

        bottom_block.render(bottom, buf);
    }
}

fn render_main_menu(area: Rect, buf: &mut Buffer, _app: &App) {
    let title = Line::from("Main Menu").bold().centered();
    let block = Block::bordered()
        .title(title.clone())
        .border_set(border::THICK);

    let body = Text::from(vec![
        Line::from("Select an option:"),
        Line::from("     [1] Load example expression 1 in CK machine, ┃ (λx.x 42) => 42"),
        Line::from("     [2] Load example expression 2 in CK machine, ┃ ((λx.λy.x 42) 99) => 42"),
        Line::from(
            "     [3] Load example expression 3 in CESK machine┃ ((((λa.(λx.(λy.a)) 99) 123) 456) 789) => 10",
        ),
        Line::from("     [E] Enter custom expression"),
        Line::from("     [Q] Quit"),
    ]);
    Paragraph::new(body).block(block).render(area, buf);
}

fn render_expression_input(area: Rect, buf: &mut Buffer, buffer: &str, error: Option<&str>) {
    let title = Line::from("Enter Expression").bold().centered();
    let block = Block::bordered()
        .title(title.clone())
        .border_set(border::THICK);

    let mut lines = vec![
        Line::from("Type an expression and hit <ENTER> to run.").italic(),
        Line::from("Press <ESC> to go back."),
        Line::from(""),
        Line::from(format!("» {}", buffer)),
    ];

    if let Some(msg) = error {
        lines.push(Line::from(""));
        lines.push(Line::from(msg.red().bold()));
    }

    Paragraph::new(Text::from(lines))
        .block(block)
        .render(area, buf);
}
