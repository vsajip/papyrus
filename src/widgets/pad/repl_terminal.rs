use crate::prelude::*;
use crate::widgets;
use azul::app_state::AppStateNoData;
use azul::default_callbacks::{DefaultCallback, DefaultCallbackId};
use azul::prelude::*;
use azul::window::FakeWindow;
use linefeed::memory::MemoryTerminal;
use std::marker::PhantomData;
use super::*;

type KickOffEvalDaemon = bool;
type HandleCb = (UpdateScreen, KickOffEvalDaemon);

pub struct PadState {
	repl: EvalState<(), linking::NoRef>,
    terminal: MemoryTerminal,
    last_terminal_string: String,
    eval_daemon_id: DaemonId,
}

impl PadState {
    pub fn new(repl: Repl<repl::Read, MemoryTerminal, (), linking::NoRef>) -> Self {
		let term = repl.terminal_inner().clone();
        Self {
			repl: EvalState::new(repl),
            terminal: term,
            last_terminal_string: String::new(),
            eval_daemon_id: DaemonId::new(),
        }
    }

    pub fn update_state_on_text_input<T: Layout + GetPadState>(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        maybe_kickoff_daemon(
            app_state,
            self.eval_daemon_id,
            self.handle_input(
                app_state.windows[window_event.window_id]
                    .get_keyboard_state()
                    .current_char?,
            ),
        )
    }

    pub fn update_state_on_vk_down<T: Layout + GetPadState>(
        &mut self,
        app_state: &mut AppStateNoData<T>,
        window_event: &mut CallbackInfo<T>,
    ) -> UpdateScreen {
        maybe_kickoff_daemon(
            app_state,
            self.eval_daemon_id,
            self.handle_vk(
                app_state.windows[window_event.window_id]
                    .get_keyboard_state()
                    .latest_virtual_keycode?,
            ),
        )
    }

    fn handle_input(&mut self, input: char) -> HandleCb {
        let mut kickoff = false;
		match self.repl.take_read() {
			Some(repl) => {
				match repl.push_input(input) {
					repl::PushResult::Read(r) => self.repl.put_read(r),
					repl::PushResult::Eval(r) => {
						kickoff = true;
						self.repl.put_eval(r.eval_async(()));
					}
				}
				(Redraw, kickoff)
			},
			None => 			(DontRedraw, kickoff)				,
		}
    }

    fn handle_vk(&mut self, vk: VirtualKeyCode) -> HandleCb {
        match vk {
            VirtualKeyCode::Back => self.handle_input('\x08'), // backspace character
            VirtualKeyCode::Tab => self.handle_input('\t'),
            VirtualKeyCode::Return => self.handle_input('\n'),
            _ => (DontRedraw, false),
        }
    }

	fn redraw_on_term_chg(&mut self) -> UpdateScreen {
		let new_str = create_terminal_string(&self.terminal);
		if new_str != self.last_terminal_string {
			self.last_terminal_string = new_str;
			Redraw
		} else {
			DontRedraw
		}
	}
}

pub trait GetPadState {
    fn pad_state(&mut self) -> &mut PadState;
}

pub struct ReplTerminal<T: Layout> {
    text_input_cb_id: DefaultCallbackId,
    vk_down_cb_id: DefaultCallbackId,
    mrkr: PhantomData<T>,
}

impl<T: Layout + GetPadState> ReplTerminal<T> {
    pub fn new(
        window: &mut FakeWindow<T>,
        state_to_bind: &PadState,
        full_data_model: &T,
    ) -> Self {
        let ptr = StackCheckedPointer::new(full_data_model, state_to_bind).unwrap();
        let text_input_cb_id =
            window.add_callback(ptr, DefaultCallback(Self::update_state_on_text_input));
        let vk_down_cb_id =
            window.add_callback(ptr, DefaultCallback(Self::update_state_on_vk_down));

        Self {
            text_input_cb_id,
            vk_down_cb_id,
            mrkr: PhantomData,
        }
    }

    pub fn dom(self, state_to_render: &PadState) -> Dom<T> {
        let term_str = create_terminal_string(&state_to_render.terminal);

        let categorised = cansi::categorise_text(&term_str);

        let mut container = Dom::div()
            .with_class("repl-terminal")
            .with_tab_index(TabIndex::Auto); // make focusable
        container.add_default_callback_id(On::TextInput, self.text_input_cb_id);
        container.add_default_callback_id(On::VirtualKeyDown, self.vk_down_cb_id);

        for line in cansi::line_iter(&categorised) {
            let mut line_div = Dom::div().with_class("repl-terminal-line");
            for cat in line {
                line_div.add_child(colour_slice(&cat));
            }
            container.add_child(line_div);
        }

        //container.debug_dump();	// debug layout

        container
    }

    cb!(PadState, update_state_on_text_input);
    cb!(PadState, update_state_on_vk_down);
}

fn maybe_kickoff_daemon<T: Layout + GetPadState>(
    app_state: &mut AppStateNoData<T>,
    daemon_id: DaemonId,
    handle_result: HandleCb,
) -> UpdateScreen {
    let (r, kickoff) = handle_result;
    if kickoff {
        let daemon =
            Daemon::new(check_evaluating_done).with_interval(std::time::Duration::from_millis(2));
        app_state.add_daemon(daemon_id, daemon);
    }

    r
}

fn check_evaluating_done<T: GetPadState>(
    app: &mut T,
    _: &mut AppResources,
) -> (UpdateScreen, TerminateDaemon) {
    let pad = app.pad_state();

	match pad.repl.take_eval() {
		Some(eval) => {
			if eval.completed() {
				pad.repl.put_read(eval.wait()
						.expect("got an eval signal, which I have not handled yet")
						.print());				
				(Redraw, TerminateDaemon::Terminate) // turn off daemon now
			} else {
				pad.repl.put_eval(eval);
				(pad.redraw_on_term_chg(), TerminateDaemon::Continue) // continue to check later
			}
		},
		None => (DontRedraw, TerminateDaemon::Terminate) // if there is no eval, may as well stop checking
	}
}

fn create_terminal_string(term: &MemoryTerminal) -> String {
    let mut string = String::new();
    let mut lines = term.lines();
    while let Some(chars) = lines.next() {
        for ch in chars {
            string.push(*ch);
        }
        string.push('\n');
    }
    string
}

fn colour_slice<T: Layout>(cat_slice: &cansi::CategorisedSlice) -> Dom<T> {
    const PROPERTY_STR: &str = "ansi_esc_color";
    let s = String::from_utf8_lossy(cat_slice.text_as_bytes);

    Dom::label(s)
        .with_class("repl-terminal-text")
        .with_css_override(
            PROPERTY_STR,
            StyleTextColor(widgets::colour::map(&cat_slice.fg_colour)).into(),
        )
}

pub const PAD_CSS: &'static str = r##"
.repl-terminal {
	background-color: black;
	padding: 5px;
}

.repl-terminal-line {
	flex-direction: row;
}

.repl-terminal-text {
	color: [[ ansi_esc_color | white ]];
	text-align: left;
	line-height: 135%;
	font-size: 1em;
	font-family: Lucida Console,Lucida Sans Typewriter,monaco,Bitstream Vera Sans Mono,monospace;
}

.repl-terminal-text:hover {
	border: 1px solid #9b9b9b;
}
"##;