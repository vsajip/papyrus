use super::*;
use proc_macro2::Span;
use syn::{self, export::ToTokens, spanned::Spanned, Block, Item, Stmt};

/// Parses a line of input as a command.
/// Returns either a `Command` value or an `InputError` value.
pub fn parse_command(line: &str) -> InputResult {
	if !is_command(line) {
		return InputResult::InputError("command must begin with `.` or `:`".to_string());
	}

	let line = &line[1..];
	let mut words = line.trim_right().splitn(2, ' ');

	match words.next() {
		Some(name) if !name.is_empty() => {
			InputResult::Command(name.to_string(), words.next().unwrap_or(&"").to_string())
		}
		_ => InputResult::InputError("expected command name".to_string()),
	}
}

/// Parses a line of input as a program.
pub fn parse_program(code: &str) -> InputResult {
	debug!("parse program: {}", code);
	let code = format!("{{ {} }}", code); // wrap in a block so the parser can parse through it without need to guess the type!

	match syn::parse_str::<Block>(&code) {
		Ok(block) => {
			let block_span = block.span();
			let block_span = MySpan::derive_from_span(block_span);
			debug!("Block Span: {:?}", block_span);
			let mut stmts = Vec::new();
			let mut items = Vec::new();
			let mut crates = Vec::new();
			for stmt in block.stmts {
				match stmt {
					Stmt::Local(local) => {
						let mut s = extract_span_str(&code, local.span(), block_span.lo);
						s.pop(); // local is handled slightly differently, the trailing semi is dropped.
						stmts.push(Statement {
							expr: s,
							semi: true,
						})
					}
					Stmt::Item(item) => match parse_item(item) {
						ParseItemResult::ExternCrate(span) => {
							match CrateType::parse_str(&extract_span_str(
								&code,
								span,
								block_span.lo,
							)) {
								Ok(c) => crates.push(c),
								Err(e) => error!("crate parsing failed: {}", e),
							}
						}
						ParseItemResult::Span(span) => {
							items.push(extract_span_str(&code, span, block_span.lo))
						}
						ParseItemResult::Error(s) => return InputResult::InputError(s),
					},
					Stmt::Expr(expr) => match parse_expr(expr) {
						Ok(span) => stmts.push(Statement {
							expr: extract_span_str(&code, span, block_span.lo),
							semi: false,
						}),
						Err(s) => return InputResult::InputError(s),
					},
					Stmt::Semi(expr, _) => match parse_expr(expr) {
						Ok(span) => stmts.push(Statement {
							expr: extract_span_str(&code, span, block_span.lo),
							semi: true,
						}),
						Err(s) => return InputResult::InputError(s),
					},
				}
			}
			InputResult::Program(Input {
				items: items,
				stmts: stmts,
				crates: crates,
			})
		}
		Err(e) => {
			if e.to_string() == "LexError" {
				InputResult::More
			} else {
				InputResult::InputError(e.to_string())
			}
		}
	}
}

fn extract_span_str(code: &str, span: Span, start_pos: u32) -> String {
	let span = MySpan::derive_from_span(span);
	debug!("Stmt Span: {:?}", span);
	let s = code[(span.lo - start_pos) as usize..(span.hi - start_pos) as usize].to_string();
	debug!("Code slice: {}", s);
	s
}

enum ParseItemResult {
	Span(Span),
	ExternCrate(Span),
	Error(String),
}

fn parse_item(item: Item) -> ParseItemResult {
	match item {
		Item::Use(_) => {
			error!("haven't handled item variant Use");
			ParseItemResult::Error("haven't handled item variant Use. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Static(_) => {
			error!("haven't handled item variant Static");
			ParseItemResult::Error("haven't handled item variant Static. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Const(_) => {
			error!("haven't handled item variant Const");
			ParseItemResult::Error("haven't handled item variant Const. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Mod(_) => {
			error!("haven't handled item variant Mod");
			ParseItemResult::Error("haven't handled item variant Mod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::ForeignMod(_) => {
			error!("haven't handled item variant ForeignMod");
			ParseItemResult::Error("haven't handled item variant ForeignMod. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Type(_) => {
			error!("haven't handled item variant Type");
			ParseItemResult::Error("haven't handled item variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Existential(_) => {
			error!("haven't handled item variant Existential");
			ParseItemResult::Error("haven't handled item variant Existential. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Enum(_) => {
			error!("haven't handled item variant Enum");
			ParseItemResult::Error("haven't handled item variant Enum. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Union(_) => {
			error!("haven't handled item variant Union");
			ParseItemResult::Error("haven't handled item variant Union. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Trait(_) => {
			error!("haven't handled item variant Trait");
			ParseItemResult::Error("haven't handled item variant Trait. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::TraitAlias(_) => {
			error!("haven't handled item variant TraitAlias");
			ParseItemResult::Error("haven't handled item variant TraitAlias. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Impl(_) => {
			error!("haven't handled item variant Impl");
			ParseItemResult::Error("haven't handled item variant Impl. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Macro(_) => {
			error!("haven't handled item variant Macro");
			ParseItemResult::Error("haven't handled item variant Macro. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Macro2(_) => {
			error!("haven't handled item variant Macro2");
			ParseItemResult::Error("haven't handled item variant Macro2. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::Verbatim(_) => {
			error!("haven't handled item variant Verbatim");
			ParseItemResult::Error("haven't handled item variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Item::ExternCrate(_) => {
			let span = item.span();
			let s = format!("{}", item.into_token_stream());
			debug!("Item parsed, its a crate: {}", s);
			ParseItemResult::ExternCrate(span)
		}
		_ => {
			let span = item.span();
			let s = format!("{}", item.into_token_stream());
			debug!("Item parsed: {}", s);
			ParseItemResult::Span(span)
		}
	}
}

fn parse_expr(expr: Expr) -> Result<Span, String> {
	match expr {
		Expr::Box(_) => {
			error!("haven't handled expr variant Box");
			Err("haven't handled expr variant Box. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::InPlace(_) => {
			error!("haven't handled expr variant InPlace");
			Err("haven't handled expr variant InPlace. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Array(_) => {
			error!("haven't handled expr variant Array");
			Err("haven't handled expr variant Array. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Unary(_) => {
			error!("haven't handled expr variant Unary");
			Err("haven't handled expr variant Unary. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Lit(_) => {
			error!("haven't handled expr variant Lit");
			Err("haven't handled expr variant Lit. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Cast(_) => {
			error!("haven't handled expr variant Cast");
			Err("haven't handled expr variant Cast. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Type(_) => {
			error!("haven't handled expr variant Type");
			Err("haven't handled expr variant Type. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::If(_) => {
			error!("haven't handled expr variant If");
			Err("haven't handled expr variant If. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::While(_) => {
			error!("haven't handled expr variant While");
			Err("haven't handled expr variant While. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Loop(_) => {
			error!("haven't handled expr variant For");
			Err("haven't handled expr variant For. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Match(_) => {
			error!("haven't handled expr variant Match");
			Err("haven't handled expr variant Match. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Closure(_) => {
			error!("haven't handled expr variant Closure");
			Err("haven't handled expr variant Closure. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Unsafe(_) => {
			error!("haven't handled expr variant Unsafe");
			Err("haven't handled expr variant Unsafe. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Block(_) => {
			error!("haven't handled expr variant Block");
			Err("haven't handled expr variant Block. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Assign(_) => {
			error!("haven't handled expr variant Assign");
			Err("haven't handled expr variant Assign. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::AssignOp(_) => {
			error!("haven't handled expr variant AssignOp");
			Err("haven't handled expr variant AssignOp. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Field(_) => {
			error!("haven't handled expr variant Field");
			Err("haven't handled expr variant Field. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Index(_) => {
			Err("haven't handled expr variant Index. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Range(_) => {
			error!("haven't handled expr variant Range");
			Err("haven't handled expr variant Range. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Reference(_) => {
			error!("haven't handled expr variant Reference");
			Err("haven't handled expr variant Reference. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Break(_) => {
			error!("haven't handled expr variant Break");
			Err("haven't handled expr variant Break. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Continue(_) => {
			error!("haven't handled expr variant Continue");
			Err("haven't handled expr variant Continue. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Return(_) => {
			error!("haven't handled expr variant Return");
			Err("haven't handled expr variant Return. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Struct(_) => {
			error!("haven't handled expr variant Struct");
			Err("haven't handled expr variant Struct. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Repeat(_) => {
			error!("haven't handled expr variant Repeat");
			Err("haven't handled expr variant Repeat. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Paren(_) => {
			error!("haven't handled expr variant Paren");
			Err("haven't handled expr variant Paren. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Group(_) => {
			error!("haven't handled expr variant Group");
			Err("haven't handled expr variant Group. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Try(_) => {
			error!("haven't handled expr variant Try");
			Err("haven't handled expr variant Try. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Async(_) => {
			error!("haven't handled expr variant Async");
			Err("haven't handled expr variant Async. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::TryBlock(_) => {
			error!("haven't handled expr variant TryBlock");
			Err("haven't handled expr variant TryBlock. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Yield(_) => {
			error!("haven't handled expr variant Yield");
			Err("haven't handled expr variant Yield. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		Expr::Verbatim(_) => {
			error!("haven't handled expr variant Verbatim");
			Err("haven't handled expr variant Verbatim. Raise a request here https://github.com/kurtlawrence/papyrus/issues".to_string())
		}
		_ => {
			let span = expr.span();
			let s = format!("{}", expr.into_token_stream());
			debug!("Expression parsed: {}", s);
			Ok(span)
		}
	}
}

#[derive(Debug)]
struct MySpan {
	lo: u32,
	hi: u32,
}

impl MySpan {
	fn derive_from_span(span: Span) -> Self {
		let s = format!("{:?}", span);
		debug!("{} len: {}", s, s.len());
		assert!(
			&s != "Span",
			"papyrus needs to be built against nightly Rust with RUSTFLAGS='--cfg procmacro2_semver_exempt'"
		);
		// bytes(#..#)
		let slice = &s[6..s.len() - 1];
		let mut ns = slice.split("..");
		MySpan {
			lo: ns.next().unwrap().parse::<u32>().unwrap(),
			hi: ns.next().unwrap().parse::<u32>().unwrap(),
		}
	}
}
