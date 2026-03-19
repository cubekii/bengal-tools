use bengal_compiler::compiler::{Compiler, CompilerOptions};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, HtmlInputElement, HtmlTextAreaElement, Window};
use yew::prelude::*;
use yew::virtual_dom::VNode;

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

    // Convert highlighted HTML strings into Yew VNodes via Html::from_html_unchecked
    let highlighted_vnode: VNode =
        Html::from_html_unchecked(AttrValue::from((*highlighted_code).clone()));
    let output_vnode: VNode =
        Html::from_html_unchecked(AttrValue::from((*formatted_output).clone()));

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
                            style="position: absolute; top: 0; left: 4rem; right: 0; bottom: 0; padding: 1rem; pointer-events: none; white-space: pre; overflow: hidden; line-height: 1.5; font-family: monospace; font-size: 14px; background: #1e1e1e;"
                        >
                            { highlighted_vnode }
                        </div>
                        <textarea
                            ref={textarea_ref}
                            style="position: absolute; top: 0; left: 4rem; right: 0; bottom: 0; padding: 1rem; background: transparent; color: transparent; caret-color: white; border: none; outline: none; resize: none; font-family: monospace; font-size: 14px; line-height: 1.5; z-index: 1;"
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
                        >
                            { output_vnode }
                        </div>
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
            let output = display_bytecode_to_string(&bytecode);
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

fn display_bytecode_to_string(bytecode: &sparkler::Bytecode) -> String {
    let mut output = String::new();
    
    output.push_str("# Bytecode Viewer - Bengal\n\n");
    
    // Display .data section (constants)
    output.push_str(".data\n");
    for (i, s) in bytecode.strings.iter().enumerate() {
        output.push_str(&format!("  str.{:<4} = \"{}\"\n", i, escape_string(s)));
    }
    for class in &bytecode.classes {
        output.push_str(&format!("  class.{} =\n", class.name));
        for (field_name, field_value) in &class.fields {
            output.push_str(&format!("    .{} = {:?}\n", field_name, field_value));
        }
    }
    output.push('\n');
    
    // Display module-level (root) code
    if !bytecode.data.is_empty() {
        output.push_str(".root:\n");
        output.push_str("# module-level code\n");
        let mut pc = 0;
        let data = &bytecode.data;
        while pc < data.len() {
            let opcode_byte = data[pc];
            let opcode = opcode_from_byte(opcode_byte);
            let address = format!("{:04x}", pc);
            let (opcode_name, operands, operand_count) = decode_instruction(data, pc, opcode, &bytecode.strings);
            if operands.is_empty() {
                output.push_str(&format!("  {} | {}\n", address, opcode_name));
            } else {
                output.push_str(&format!("  {} | {:<18} | {}\n", address, opcode_name, operands));
            }
            pc += 1 + operand_count;
        }
        output.push('\n');
    }
    
    // Display functions
    for function in &bytecode.functions {
        output.push_str(&format!("{}:\n", function.name));
        output.push_str(&format!("# registers: {}, source: {:?}\n", function.register_count, function.source_file));
        let mut pc = 0;
        let data = &function.bytecode;
        while pc < data.len() {
            let opcode_byte = data[pc];
            let opcode = opcode_from_byte(opcode_byte);
            let address = format!("{:04x}", pc);
            let (opcode_name, operands, operand_count) = decode_instruction(data, pc, opcode, &bytecode.strings);
            if operands.is_empty() {
                output.push_str(&format!("  {} | {}\n", address, opcode_name));
            } else {
                output.push_str(&format!("  {} | {:<18} | {}\n", address, opcode_name, operands));
            }
            pc += 1 + operand_count;
        }
        output.push('\n');
    }
    
    output
}

fn opcode_from_byte(byte: u8) -> sparkler::Opcode {
    use sparkler::Opcode;
    match byte {
        0x00 => Opcode::Nop,
        0x10 => Opcode::LoadConst,
        0x11 => Opcode::LoadInt,
        0x12 => Opcode::LoadFloat,
        0x13 => Opcode::LoadBool,
        0x14 => Opcode::LoadNull,
        0x20 => Opcode::Move,
        0x21 => Opcode::LoadLocal,
        0x22 => Opcode::StoreLocal,
        0x30 => Opcode::GetProperty,
        0x31 => Opcode::SetProperty,
        0x40 => Opcode::Call,
        0x41 => Opcode::CallNative,
        0x42 => Opcode::Invoke,
        0x43 => Opcode::Return,
        0x44 => Opcode::CallAsync,
        0x45 => Opcode::CallNativeAsync,
        0x46 => Opcode::InvokeAsync,
        0x47 => Opcode::Await,
        0x48 => Opcode::Spawn,
        0x49 => Opcode::InvokeInterface,
        0x4A => Opcode::InvokeInterfaceAsync,
        0x4B => Opcode::CallNativeIndexed,
        0x4C => Opcode::CallNativeIndexedAsync,
        0x50 => Opcode::Jump,
        0x51 => Opcode::JumpIfTrue,
        0x52 => Opcode::JumpIfFalse,
        0x60 => Opcode::Equal,
        0x61 => Opcode::NotEqual,
        0x62 => Opcode::And,
        0x63 => Opcode::Or,
        0x64 => Opcode::Not,
        0x65 => Opcode::Concat,
        0x66 => Opcode::Greater,
        0x67 => Opcode::Less,
        0x68 => Opcode::Add,
        0x69 => Opcode::Subtract,
        0x6A => Opcode::GreaterEqual,
        0x6B => Opcode::LessEqual,
        0x70 => Opcode::Multiply,
        0x71 => Opcode::Divide,
        0x74 => Opcode::Convert,
        0x75 => Opcode::Modulo,
        0x76 => Opcode::Array,
        0x77 => Opcode::Index,
        0x73 => Opcode::Line,
        0x78 => Opcode::BitAnd,
        0x79 => Opcode::BitOr,
        0x7A => Opcode::BitXor,
        0x7B => Opcode::BitNot,
        0x7C => Opcode::ShiftLeft,
        0x7D => Opcode::ShiftRight,
        0x80 => Opcode::TryStart,
        0x81 => Opcode::TryEnd,
        0x82 => Opcode::Throw,
        0x90 => Opcode::Breakpoint,
        0xFF => Opcode::Halt,
        _ => Opcode::Nop,
    }
}

fn decode_instruction(data: &[u8], pc: usize, opcode: sparkler::Opcode, strings: &[String]) -> (String, String, usize) {
    match opcode {
        sparkler::Opcode::Nop => ("NOP".to_string(), String::new(), 0),
        sparkler::Opcode::LoadConst => {
            if pc + 2 < data.len() {
                let str_idx = data[pc + 2] as usize;
                let value = strings.get(str_idx)
                    .map(|s| format!("\"{}\"", escape_string(s)))
                    .unwrap_or_else(|| format!("str.{}", str_idx));
                (format!("LOAD_CONST R{}", data[pc + 1]), value, 2)
            } else {
                ("LOAD_CONST".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::LoadInt => {
            if pc + 10 <= data.len() {
                let value = i64::from_le_bytes([
                    data[pc + 2], data[pc + 3], data[pc + 4], data[pc + 5],
                    data[pc + 6], data[pc + 7], data[pc + 8], data[pc + 9],
                ]);
                (format!("LOAD_INT R{}", data[pc + 1]), format!("{}", value), 9)
            } else {
                ("LOAD_INT".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::LoadFloat => {
            if pc + 10 <= data.len() {
                let value = f64::from_le_bytes([
                    data[pc + 2], data[pc + 3], data[pc + 4], data[pc + 5],
                    data[pc + 6], data[pc + 7], data[pc + 8], data[pc + 9],
                ]);
                (format!("LOAD_FLOAT R{}", data[pc + 1]), format!("{}", value), 9)
            } else {
                ("LOAD_FLOAT".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::LoadBool => {
            if pc + 2 < data.len() {
                let value = data[pc + 2] != 0;
                (format!("LOAD_BOOL R{}", data[pc + 1]), format!("{}", value), 2)
            } else {
                ("LOAD_BOOL".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::LoadNull => {
            if pc + 1 < data.len() {
                (format!("LOAD_NULL R{}", data[pc + 1]), String::new(), 1)
            } else {
                ("LOAD_NULL".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::Move => {
            if pc + 2 < data.len() {
                (format!("MOVE R{}, R{}", data[pc + 1], data[pc + 2]), String::new(), 2)
            } else {
                ("MOVE".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::LoadLocal => {
            if pc + 2 < data.len() {
                let name_idx = data[pc + 2] as usize;
                let name = strings.get(name_idx)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format!("str.{}", name_idx));
                (format!("LOAD_LOCAL R{}", data[pc + 1]), format!("\"{}\"", name), 2)
            } else {
                ("LOAD_LOCAL".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::StoreLocal => {
            if pc + 2 < data.len() {
                let name_idx = data[pc + 1] as usize;
                let name = strings.get(name_idx)
                    .map(|s| s.clone())
                    .unwrap_or_else(|| format!("str.{}", name_idx));
                (format!("STORE_LOCAL R{}", data[pc + 2]), format!("\"{}\"", name), 2)
            } else {
                ("STORE_LOCAL".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::Return => {
            if pc + 1 < data.len() {
                (format!("RETURN R{}", data[pc + 1]), String::new(), 1)
            } else {
                ("RETURN".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::Jump => {
            if pc + 2 < data.len() {
                let target = u16::from_le_bytes([data[pc + 1], data[pc + 2]]);
                (format!("JUMP"), format!("-> {:04x}", target), 2)
            } else {
                ("JUMP".to_string(), String::new(), 0)
            }
        }
        sparkler::Opcode::JumpIfFalse => {
            if pc + 3 < data.len() {
                let target = u16::from_le_bytes([data[pc + 2], data[pc + 3]]);
                (format!("JUMP_IF_FALSE R{}", data[pc + 1]), format!("-> {:04x}", target), 3)
            } else {
                ("JUMP_IF_FALSE".to_string(), String::new(), 0)
            }
        }
        _ => (format!("{:?}", opcode), String::new(), 0)
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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
                    if c.is_alphanumeric() || c == '_' || c == ':' || c == '.' {
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