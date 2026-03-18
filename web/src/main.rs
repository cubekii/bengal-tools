mod bytecode_viewer;

use bengal_compiler::compiler::{Compiler, CompilerOptions};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, HtmlInputElement, HtmlTextAreaElement, Window};
use yew::prelude::*;

#[function_component]
fn App() -> Html {
    let source = use_state(|| String::from("import std.io\nprintln(\"hello world!\")"));
    let output = use_state(|| String::new());
    let unsafe_fast = use_state(|| false);
    let is_compiling = use_state(|| false);

    let textarea_ref = use_node_ref();
    let highlight_ref = use_node_ref();
    let line_numbers_ref = use_node_ref();

    // Sync scroll between textarea, highlight div, and line numbers
    use_effect_with(
        (textarea_ref.clone(), highlight_ref.clone(), line_numbers_ref.clone()),
        move |(textarea_ref, highlight_ref, line_numbers_ref)| {
            if let (Some(textarea_el), Some(highlight_el), Some(line_numbers_el)) = (
                textarea_ref.cast::<HtmlTextAreaElement>(),
                highlight_ref.cast::<HtmlDivElement>(),
                line_numbers_ref.cast::<HtmlDivElement>(),
            ) {
                let textarea_for_scroll: HtmlTextAreaElement = textarea_el.clone();
                let highlight_for_scroll: HtmlDivElement = highlight_el.clone();
                let line_numbers_for_scroll: HtmlDivElement = line_numbers_el.clone();

                let on_scroll = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
                    highlight_for_scroll.set_scroll_top(textarea_for_scroll.scroll_top());
                    highlight_for_scroll.set_scroll_left(textarea_for_scroll.scroll_left());
                    line_numbers_for_scroll.set_scroll_top(textarea_for_scroll.scroll_top());
                });

                textarea_el
                    .add_event_listener_with_callback("scroll", on_scroll.as_ref().unchecked_ref())
                    .unwrap();
                on_scroll.forget(); // Keep closure alive
            }

            || {}
        },
    );

    // Compile on source change with debounce
    {
        let source = source.clone();
        let output = output.clone();
        let unsafe_fast = unsafe_fast.clone();
        let is_compiling = is_compiling.clone();

        use_effect_with((source.clone(), unsafe_fast.clone()), move |(source, unsafe_fast)| {
            let output = output.clone();
            let is_compiling = is_compiling.clone();
            let source_val = (**source).clone();
            let unsafe_fast_val = **unsafe_fast;

            is_compiling.set(true);

            let timeout_id = set_timeout(
                move || {
                    let result = compile_source(&source_val, unsafe_fast_val);
                    output.set(result);
                    is_compiling.set(false);
                },
                300,
            );

            move || {
                clear_timeout(timeout_id);
            }
        });
    }

    let on_source_change = {
        let source = source.clone();

        Callback::from(move |e: InputEvent| {
            let target: HtmlTextAreaElement = e.target().unwrap().dyn_into().unwrap();
            source.set(target.value());
        })
    };

    let on_unsafe_change = {
        let unsafe_fast = unsafe_fast.clone();

        Callback::from(move |e: Event| {
            let target: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
            unsafe_fast.set(target.checked());
        })
    };

    let highlighted_code = use_memo((source.clone(),), |(source,)| highlight_code(source));

    let line_numbers = use_memo((source.clone(),), |(source,)| {
        let lines = source.split('\n').count();
        (1..=lines).map(|i| i.to_string()).collect::<Vec<_>>().join("\n")
    });

    let formatted_output = use_memo((output.clone(),), |(output,)| format_output(output));

    html! {
        <div style="display: flex; flex-direction: column; height: 100vh; font-family: monospace;">
            <style>{r#"
                .keyword { color: #c586c0; font-weight: bold; }
                .keyword-operator { color: #d4d4d4; }
                .keyword-import { color: #c586c0; font-weight: bold; }
                .string { color: #ce9178; }
                .number-float { color: #b5cea8; }
                .number-int { color: #b5cea8; }
                .comment { color: #6a9955; font-style: italic; }
                .comment-block { color: #6a9955; font-style: italic; }
                .type { color: #4ec9b0; }
                .primitive-type { color: #4ec9b0; font-weight: bold; }
                .function { color: #dcdcaa; }
                .constant { color: #569cd6; font-weight: bold; }
                .interpolation { color: #d4d4d4; }
                .interpolation-punct { color: #c586c0; }
                .module-path { color: #4ec9b0; }
                .variable { color: #9cdcfe; }
                .text { color: #d4d4d4; }
                .error { color: #f48771; }
            "#}</style>
            <header style="display: flex; justify-content: space-between; align-items: center; padding: 1rem; background: #1e1e1e; color: white;">
                <h1 style="margin: 0; font-size: 1.5rem;">{ "🎇 Bengal Compiler Explorer" }</h1>
                <div class="options">
                    <label class="checkbox-label" style="display: flex; align-items: center; gap: 0.5rem;">
                        <input
                            type="checkbox"
                            checked={*unsafe_fast}
                            onchange={on_unsafe_change}
                        />
                        { "Unsafe Fast" }
                    </label>
                </div>
            </header>

            <div style="display: flex; flex: 1; overflow: hidden;">
                <div style="display: flex; flex-direction: column; flex: 1; padding: 0.5rem;">
                    <div style="padding: 0.5rem; background: #2d2d2d; color: #ccc; font-weight: bold;">{ "Source Code" }</div>
                    <div style="display: flex; flex: 1; overflow: hidden; position: relative;">
                        <div
                            ref={line_numbers_ref}
                            style="padding: 1rem; background: #1e1e1e; color: #666; text-align: right; user-select: none; min-width: 3rem; white-space: pre; line-height: 1.5; font-family: monospace; font-size: 14px;"
                        >
                            { (*line_numbers).clone() }
                        </div>
                        <div
                            ref={highlight_ref}
                            style="position: absolute; top: 0; left: 4rem; right: 0; bottom: 0; padding: 1rem; color: #ffffff; pointer-events: none; white-space: pre; overflow: hidden;"
                            dangerously_set_inner_html={(*highlighted_code).clone()}
                        ></div>
                        <textarea
                            ref={textarea_ref}
                            style="position: absolute; top: 0; left: 4rem; right: 0; bottom: 0; padding: 1rem; background: transparent; color: white; caret-color: white; border: none; outline: none; resize: none; font-family: monospace; font-size: 14px; line-height: 1.5;"
                            value={(*source).clone()}
                            oninput={on_source_change}
                            spellcheck="false"
                            placeholder="Enter Bengal code here..."
                        />
                    </div>
                </div>

                <div style="display: flex; flex-direction: column; flex: 1; padding: 0.5rem; border-left: 1px solid #333;">
                    <div style="padding: 0.5rem; background: #2d2d2d; color: #ccc; font-weight: bold;">{ "Bytecode Output" }</div>
                    <div style="flex: 1; overflow: auto; background: #1e1e1e; padding: 1rem;">
                        <div
                            class="assembly-output"
                            style="color: #d4d4d4; white-space: pre-wrap; font-family: monospace; font-size: 14px; line-height: 1.5;"
                            dangerously_set_inner_html={(*formatted_output).clone()}
                        >{formatted_output}</div>
                    </div>
                </div>
            </div>
            if *is_compiling {
                <div style="position: fixed; bottom: 1rem; right: 1rem; background: #007acc; color: white; padding: 0.5rem 1rem; border-radius: 4px;">{ "Compiling..." }</div>
            }
        </div>
    }
}

fn compile_source(source: &str, unsafe_fast: bool) -> String {
    let mut compiler = Compiler::with_path_and_options(source, "<input>", unsafe_fast);
    compiler.enable_type_checking = false;

    let options = CompilerOptions {
        enable_type_checking: false,
        search_paths: vec![],
        unsafe_fast,
    };

    match compiler.compile_with_options(&options) {
        Ok(bytecode) => {
            let output = bytecode_viewer::display_bytecode(&bytecode);
            if output.is_empty() {
                format!("# Compilation succeeded but no bytecode generated\n# Source: {} bytes", source.len())
            } else {
                output
            }
        }
        Err(e) => {
            let error_msg: String = e.to_string();
            format!("Compilation Error:\n{}", error_msg)
        }
    }
}

fn format_output(output: &str) -> String {
    if output.is_empty() {
        return String::from("# No output generated");
    }
    escape_html(output)
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[derive(Debug, Clone, PartialEq)]
enum TokenType {
    Keyword,
    KeywordOperator,
    KeywordImport,
    String,
    NumberFloat,
    NumberInt,
    Comment,
    CommentBlock,
    Type,
    PrimitiveType,
    Function,
    Constant,
    Interpolation,
    ModulePath,
    Variable,
    Text,
}

impl TokenType {
    fn class_name(&self) -> &'static str {
        match self {
            TokenType::Keyword => "keyword",
            TokenType::KeywordOperator => "keyword-operator",
            TokenType::KeywordImport => "keyword-import",
            TokenType::String => "string",
            TokenType::NumberFloat => "number-float",
            TokenType::NumberInt => "number-int",
            TokenType::Comment => "comment",
            TokenType::CommentBlock => "comment-block",
            TokenType::Type => "type",
            TokenType::PrimitiveType => "primitive-type",
            TokenType::Function => "function",
            TokenType::Constant => "constant",
            TokenType::Interpolation => "interpolation",
            TokenType::ModulePath => "module-path",
            TokenType::Variable => "variable",
            TokenType::Text => "text",
        }
    }
}

#[derive(Debug, Clone)]
struct Token {
    token_type: TokenType,
    text: String,
    raw: bool,
}

fn highlight_code(code: &str) -> String {
    let tokens = tokenize(code);
    build_html(&tokens)
}

fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut pos = 0;
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();

    while pos < len {
        if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '/' {
            let mut end = pos + 2;
            while end < len && chars[end] != '\n' {
                end += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Comment,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
            let mut end = pos + 2;
            while end + 1 < len {
                if chars[end] == '*' && chars[end + 1] == '/' {
                    end += 2;
                    break;
                }
                end += 1;
            }
            if end >= len {
                end = len;
            }
            tokens.push(Token {
                token_type: TokenType::CommentBlock,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        if pos + 3 <= len && &code[pos..pos + 3] == "\"\"\"" {
            let mut end = pos + 3;
            while end + 3 <= len {
                if &code[end..end + 3] == "\"\"\"" {
                    end += 3;
                    break;
                }
                end += 1;
            }
            if end >= len {
                end = len;
            }
            tokens.push(Token {
                token_type: TokenType::String,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        if chars[pos] == '"' {
            let mut end = pos + 1;
            while end < len {
                if chars[end] == '\\' && end + 1 < len {
                    end += 2;
                    continue;
                }
                if chars[end] == '"' {
                    end += 1;
                    break;
                }
                end += 1;
            }
            tokens.push(Token {
                token_type: TokenType::String,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        if chars[pos] == '$' && pos + 1 < len && chars[pos + 1] == '{' {
            let mut depth = 1;
            let mut end = pos + 2;
            while end < len && depth > 0 {
                if chars[end] == '{' {
                    depth += 1;
                } else if chars[end] == '}' {
                    depth -= 1;
                }
                end += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Interpolation,
                text: highlight_interpolation(&code[pos..end]),
                raw: true,
            });
            pos = end;
            continue;
        }

        if pos == 0 || chars[pos - 1].is_whitespace() {
            if let Some(kw_match) = find_import_or_module(&code[pos..]) {
                let (keyword, module_path, total_len) = kw_match;
                tokens.push(Token {
                    token_type: TokenType::KeywordImport,
                    text: keyword.clone(),
                    raw: false,
                });

                let mut ws_pos = pos + keyword.len();
                while ws_pos < pos + total_len && code.chars().nth(ws_pos).unwrap().is_whitespace() {
                    tokens.push(Token {
                        token_type: TokenType::Text,
                        text: code.chars().nth(ws_pos).unwrap().to_string(),
                        raw: false,
                    });
                    ws_pos += 1;
                }

                tokens.push(Token {
                    token_type: TokenType::ModulePath,
                    text: module_path,
                    raw: false,
                });
                pos += total_len;
                continue;
            }
        }

        if chars[pos].is_whitespace() {
            let mut end = pos + 1;
            while end < len && chars[end].is_whitespace() {
                end += 1;
            }
            tokens.push(Token {
                token_type: TokenType::Text,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        if chars[pos].is_alphabetic() || chars[pos] == '_' {
            let mut end = pos + 1;
            while end < len && (chars[end].is_alphanumeric() || chars[end] == '_') {
                end += 1;
            }
            let word = &code[pos..end];

            let mut after = end;
            while after < len && chars[after].is_whitespace() {
                after += 1;
            }
            let next_char = chars.get(after).copied();

            if is_keyword(word) {
                tokens.push(Token {
                    token_type: TokenType::Keyword,
                    text: word.to_string(),
                    raw: false,
                });
            } else if word == "true" || word == "false" || word == "null" {
                tokens.push(Token {
                    token_type: TokenType::Constant,
                    text: word.to_string(),
                    raw: false,
                });
            } else if is_primitive_type(word) {
                tokens.push(Token {
                    token_type: TokenType::PrimitiveType,
                    text: word.to_string(),
                    raw: false,
                });
            } else if word.chars().next().unwrap().is_uppercase() && next_char != Some('(') {
                tokens.push(Token {
                    token_type: TokenType::Type,
                    text: word.to_string(),
                    raw: false,
                });
            } else if next_char == Some('(') && !word.chars().next().unwrap().is_uppercase() {
                tokens.push(Token {
                    token_type: TokenType::Function,
                    text: word.to_string(),
                    raw: false,
                });
            } else {
                tokens.push(Token {
                    token_type: TokenType::Variable,
                    text: word.to_string(),
                    raw: false,
                });
            }
            pos = end;
            continue;
        }

        if chars[pos].is_ascii_digit() {
            let mut end = pos;
            let mut has_dot = false;
            while end < len {
                if chars[end].is_ascii_digit() {
                    end += 1;
                } else if chars[end] == '.'
                    && !has_dot
                    && end + 1 < len
                    && chars[end + 1].is_ascii_digit()
                {
                    has_dot = true;
                    end += 1;
                } else {
                    break;
                }
            }
            let token_type = if has_dot {
                TokenType::NumberFloat
            } else {
                TokenType::NumberInt
            };
            tokens.push(Token {
                token_type,
                text: code[pos..end].to_string(),
                raw: false,
            });
            pos = end;
            continue;
        }

        let op = chars[pos];
        if pos + 1 < len {
            let two_char: String = chars[pos..pos + 2].iter().collect();
            if [
                "==", "!=", "<=", ">=", "&&", "||", "<<", ">>", "->", "=>", "::",
            ]
            .contains(&two_char.as_str())
            {
                tokens.push(Token {
                    token_type: TokenType::KeywordOperator,
                    text: two_char,
                    raw: false,
                });
                pos += 2;
                continue;
            }
        }

        if ['+', '-', '*', '/', '%', '=', '<', '>', '!', '&', '|', '^', '.', ',', ';', ':', '(', ')', '[', ']', '{', '}', '?'].contains(&op)
        {
            tokens.push(Token {
                token_type: TokenType::KeywordOperator,
                text: op.to_string(),
                raw: false,
            });
            pos += 1;
            continue;
        }

        tokens.push(Token {
            token_type: TokenType::Text,
            text: chars[pos].to_string(),
            raw: false,
        });
        pos += 1;
    }

    tokens
}

fn find_import_or_module(code: &str) -> Option<(String, String, usize)> {
    let keywords = ["import", "module"];
    for &kw in &keywords {
        if code.starts_with(kw) {
            let rest = &code[kw.len()..];
            if rest.starts_with(char::is_whitespace) {
                let trimmed = rest.trim_start();
                let mut path_end = 0;
                for (i, c) in trimmed.char_indices() {
                    if c.is_alphanumeric() || c == '_' || c == ':' {
                        path_end = i + 1;
                    } else {
                        break;
                    }
                }
                if path_end > 0 {
                    let module_path = trimmed[..path_end].to_string();
                    let total_len = kw.len() + (rest.len() - trimmed.len()) + path_end;
                    return Some((kw.to_string(), module_path, total_len));
                }
            }
        }
    }
    None
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "if" | "else"
            | "while"
            | "for"
            | "return"
            | "break"
            | "try"
            | "catch"
            | "throw"
            | "private"
            | "continue"
            | "match"
            | "native"
            | "fn"
            | "let"
            | "async"
            | "await"
            | "const"
            | "as"
            | "class"
            | "enum"
            | "use"
            | "self"
    )
}

fn is_primitive_type(word: &str) -> bool {
    matches!(word, "int" | "float" | "bool" | "char" | "str" | "void" | "any")
}

fn highlight_interpolation(s: &str) -> String {
    if s.len() > 2 {
        let content = &s[2..s.len() - 1];
        let highlighted_content = highlight_code(content);
        format!(
            "<span class=\"interpolation-punct\">${{</span>{}<span class=\"interpolation-punct\">}}</span>",
            highlighted_content
        )
    } else {
        escape_html(s)
    }
}

fn build_html(tokens: &[Token]) -> String {
    let mut html = String::new();
    for token in tokens {
        if token.token_type == TokenType::Text {
            html.push_str(&escape_html(&token.text));
        } else if token.raw {
            html.push_str(&token.text);
        } else {
            html.push_str(&format!(
                "<span class=\"{}\">{}</span>",
                token.token_type.class_name(),
                escape_html(&token.text)
            ));
        }
    }
    html
}

fn set_timeout<F>(callback: F, delay: u32) -> i32
where
    F: FnMut() + 'static,
{
    let window: Window = web_sys::window().expect("no global `window` exists");
    let closure = Closure::once(callback);
    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay as i32,
        )
        .expect("should register `setTimeout`");
    closure.forget();
    id
}

fn clear_timeout(id: i32) {
    let window: Window = web_sys::window().expect("no global `window` exists");
    window.clear_timeout_with_handle(id);
}

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
