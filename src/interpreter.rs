use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::context::Context;
use crate::control::{CompiledWord, ControlMap};
use crate::dictionary::{Dictionary, Word};
use crate::error::{CallFrame, ForthError, SourceLocation};
use crate::input::InputState;
use crate::lexer::{Token, TokenKind, tokenize};
use crate::time::TimeState;

#[derive(Debug, Clone)]
struct LoopFrame {
    exec_id: u64,
    do_index: usize,
    loop_index: usize,
    index: i64,
    limit: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Normal,
    Exit,
    Leave,
}

#[derive(Debug, Clone, Copy)]
struct Step {
    flow: Flow,
    advance: bool,
}

impl Step {
    fn normal() -> Self {
        Self {
            flow: Flow::Normal,
            advance: true,
        }
    }

    fn jump() -> Self {
        Self {
            flow: Flow::Normal,
            advance: false,
        }
    }

    fn flow(flow: Flow) -> Self {
        Self {
            flow,
            advance: false,
        }
    }
}

pub struct Interpreter {
    pub context: Context,
    pub dictionary: Dictionary,
    pub source_path: Option<PathBuf>,
    pub loaded_files: Vec<PathBuf>,
    include_stack: Vec<PathBuf>,
    call_stack: Vec<CallFrame>,
    loop_stack: Vec<LoopFrame>,
    next_exec_id: u64,
    recursion_depth: usize,
    max_recursion_depth: usize,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            dictionary: Dictionary::new(),
            source_path: None,
            loaded_files: Vec::new(),
            include_stack: Vec::new(),
            call_stack: Vec::new(),
            loop_stack: Vec::new(),
            next_exec_id: 1,
            recursion_depth: 0,
            max_recursion_depth: 4096,
        }
    }

    /// Tokenize and run a source file's top-level code, then call `init` once.
    pub fn load(&mut self, source: &str, path: Option<PathBuf>) -> Result<(), ForthError> {
        self.source_path = path.clone();

        let file_name = path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "<input>".to_string());

        let tokens = tokenize(source, &file_name)?;
        let compiled = CompiledWord::new("<top-level>", tokens)?;

        let canonical = path.as_ref().and_then(|path| path.canonicalize().ok());

        if let Some(canonical) = &canonical {
            self.include_stack.push(canonical.clone());
            self.remember_file(canonical);
        }

        let result = self.execute_compiled(&compiled);

        if canonical.is_some() {
            self.include_stack.pop();
        }

        match result? {
            Flow::Normal | Flow::Exit => {}
            Flow::Leave => return Err(ForthError::new("leave outside loop")),
        }

        if self.dictionary.is_user_defined("init") {
            match self.call_named_word("init", None)? {
                Flow::Normal | Flow::Exit => {}
                Flow::Leave => return Err(ForthError::new("leave outside loop in init")),
            }
        }

        Ok(())
    }

    /// Load a file from disk and run its lifecycle.
    pub fn load_file(&mut self, path: &Path) -> Result<(), ForthError> {
        let source = std::fs::read_to_string(path).map_err(|error| {
            ForthError::new(format!("cannot read '{}': {}", path.display(), error))
        })?;
        self.load(&source, Some(path.to_path_buf()))
    }

    /// Execute a source string once. This is mainly useful for tests and tools.
    pub fn eval(&mut self, source: &str) -> Result<(), ForthError> {
        let tokens = tokenize(source, "<eval>")?;
        let compiled = CompiledWord::new("<eval>", tokens)?;

        match self.execute_compiled(&compiled)? {
            Flow::Normal | Flow::Exit => Ok(()),
            Flow::Leave => Err(ForthError::new("leave outside loop")),
        }
    }

    pub fn set_time(&mut self, dt_ms: i64, elapsed_ms: i64, frame: i64) {
        self.context.time = TimeState {
            dt_ms,
            elapsed_ms,
            frame,
        };
    }

    pub fn set_input(&mut self, input: InputState) {
        self.context.input = input;
    }

    /// Run one frame: clear draw commands, then call `update` and `draw`.
    pub fn run_frame(&mut self) -> Result<(), ForthError> {
        self.context.graphics.clear();

        match self.call_named_word("update", None)? {
            Flow::Normal | Flow::Exit => {}
            Flow::Leave => return Err(ForthError::new("leave outside loop in update")),
        }

        match self.call_named_word("draw", None)? {
            Flow::Normal | Flow::Exit => {}
            Flow::Leave => return Err(ForthError::new("leave outside loop in draw")),
        }

        Ok(())
    }

    fn remember_file(&mut self, path: &Path) {
        if !self.loaded_files.iter().any(|loaded| loaded == path) {
            self.loaded_files.push(path.to_path_buf());
        }
    }

    fn execute_compiled(&mut self, word: &CompiledWord) -> Result<Flow, ForthError> {
        self.recursion_depth += 1;

        if self.recursion_depth > self.max_recursion_depth {
            self.recursion_depth -= 1;
            return Err(ForthError::new(format!(
                "recursion limit exceeded ({})",
                self.max_recursion_depth
            )));
        }

        let result = self.execute_tokens(&word.tokens, &word.control, Some(word));
        self.recursion_depth -= 1;
        result
    }

    fn execute_tokens(
        &mut self,
        tokens: &[Token],
        control: &ControlMap,
        current: Option<&CompiledWord>,
    ) -> Result<Flow, ForthError> {
        let exec_id = self.next_exec_id;
        self.next_exec_id = self.next_exec_id.wrapping_add(1);

        let loop_base = self.loop_stack.len();
        let result = self.execute_inner(tokens, control, current, exec_id);
        self.loop_stack.truncate(loop_base);
        result
    }

    fn execute_inner(
        &mut self,
        tokens: &[Token],
        control: &ControlMap,
        current: Option<&CompiledWord>,
        exec_id: u64,
    ) -> Result<Flow, ForthError> {
        let mut pc = 0;

        while pc < tokens.len() {
            let token = &tokens[pc];

            let step = match self.execute_token(token, tokens, control, current, exec_id, &mut pc) {
                Ok(step) => step,
                Err(mut error) => {
                    if error.location.is_none() {
                        error.location = Some(token.location.clone());
                    }
                    return Err(error);
                }
            };

            match step.flow {
                Flow::Normal => {
                    if step.advance {
                        pc += 1;
                    }
                }

                Flow::Leave => {
                    if let Some(frame) = self.loop_stack.last() {
                        if frame.exec_id == exec_id {
                            let loop_index = frame.loop_index;
                            self.loop_stack.pop();
                            pc = loop_index + 1;
                            continue;
                        }
                    }

                    return Ok(Flow::Leave);
                }

                Flow::Exit => return Ok(Flow::Exit),
            }
        }

        Ok(Flow::Normal)
    }

    fn execute_token(
        &mut self,
        token: &Token,
        tokens: &[Token],
        control: &ControlMap,
        current: Option<&CompiledWord>,
        exec_id: u64,
        pc: &mut usize,
    ) -> Result<Step, ForthError> {
        match &token.kind {
            TokenKind::Number(number) => {
                self.context.push(*number);
                Ok(Step::normal())
            }

            TokenKind::String(value) => {
                let handle = self.context.strings.intern(value.clone());
                self.context.push(handle);
                Ok(Step::normal())
            }

            TokenKind::Word(word) => {
                self.execute_word_token(word, token, tokens, control, current, exec_id, pc)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_word_token(
        &mut self,
        word: &str,
        token: &Token,
        tokens: &[Token],
        control: &ControlMap,
        current: Option<&CompiledWord>,
        exec_id: u64,
        pc: &mut usize,
    ) -> Result<Step, ForthError> {
        match word {
            "if" => {
                let current_pc = *pc;
                let condition = self.context.pop()?;

                if condition == 0 {
                    let target = if let Some(else_index) = control.if_else.get(&current_pc) {
                        else_index + 1
                    } else if let Some(then_index) = control.if_then.get(&current_pc) {
                        then_index + 1
                    } else {
                        return Err(ForthError::new("if without then"));
                    };

                    *pc = target;
                    return Ok(Step::jump());
                }

                Ok(Step::normal())
            }

            "else" => {
                let current_pc = *pc;
                let target = control
                    .else_then
                    .get(&current_pc)
                    .copied()
                    .ok_or_else(|| ForthError::new("else without then"))?
                    + 1;
                *pc = target;
                Ok(Step::jump())
            }

            "then" => Ok(Step::normal()),

            "begin" => Ok(Step::normal()),

            "until" => {
                let current_pc = *pc;
                let condition = self.context.pop()?;

                if condition == 0 {
                    let begin = control
                        .until_begin
                        .get(&current_pc)
                        .copied()
                        .ok_or_else(|| ForthError::new("until without begin"))?;
                    *pc = begin + 1;
                    return Ok(Step::jump());
                }

                Ok(Step::normal())
            }

            "while" => {
                let current_pc = *pc;
                let condition = self.context.pop()?;

                if condition == 0 {
                    let repeat = control
                        .while_repeat
                        .get(&current_pc)
                        .copied()
                        .ok_or_else(|| ForthError::new("while without repeat"))?;
                    *pc = repeat + 1;
                    return Ok(Step::jump());
                }

                Ok(Step::normal())
            }

            "repeat" => {
                let current_pc = *pc;
                let begin = control
                    .repeat_begin
                    .get(&current_pc)
                    .copied()
                    .ok_or_else(|| ForthError::new("repeat without begin"))?;
                *pc = begin + 1;
                Ok(Step::jump())
            }

            "do" | "?do" => {
                let current_pc = *pc;
                let start = self.context.pop()?;
                let limit = self.context.pop()?;
                let loop_index = control
                    .do_loop
                    .get(&current_pc)
                    .copied()
                    .ok_or_else(|| ForthError::new("do without loop"))?;

                if word == "?do" && start == limit {
                    *pc = loop_index + 1;
                    return Ok(Step::jump());
                }

                self.loop_stack.push(LoopFrame {
                    exec_id,
                    do_index: current_pc,
                    loop_index,
                    index: start,
                    limit,
                });

                Ok(Step::normal())
            }

            "loop" | "+loop" => {
                let increment = if word == "+loop" {
                    Some(self.context.pop()?)
                } else {
                    None
                };

                let Some(frame_index) = self.loop_stack.len().checked_sub(1) else {
                    return Err(ForthError::new("loop without do"));
                };

                let (do_index, keep_going) = {
                    let frame = &mut self.loop_stack[frame_index];

                    if frame.exec_id != exec_id {
                        return Err(ForthError::new("loop without matching do in this word"));
                    }

                    match increment {
                        Some(increment) => {
                            if increment == 0 {
                                return Err(ForthError::new("zero increment in +loop"));
                            }

                            frame.index = frame.index.wrapping_add(increment);
                            let keep_going = if increment > 0 {
                                frame.index < frame.limit
                            } else {
                                frame.index > frame.limit
                            };
                            (frame.do_index, keep_going)
                        }

                        None => {
                            frame.index = frame.index.wrapping_add(1);
                            (frame.do_index, frame.index < frame.limit)
                        }
                    }
                };

                if keep_going {
                    *pc = do_index + 1;
                    return Ok(Step::jump());
                }

                self.loop_stack.pop();
                Ok(Step::normal())
            }

            "leave" => Ok(Step::flow(Flow::Leave)),

            "i" => {
                let index = self
                    .loop_stack
                    .last()
                    .map(|frame| frame.index)
                    .ok_or_else(|| ForthError::new("i outside loop"))?;
                self.context.push(index);
                Ok(Step::normal())
            }

            "j" => {
                let depth = self.loop_stack.len();

                if depth < 2 {
                    return Err(ForthError::new("j outside a nested loop"));
                }

                let index = self.loop_stack[depth - 2].index;
                self.context.push(index);
                Ok(Step::normal())
            }

            "exit" => Ok(Step::flow(Flow::Exit)),

            "recurse" => {
                let Some(current) = current else {
                    return Err(ForthError::new("recurse outside a definition"));
                };

                let flow = self.execute_compiled(current)?;

                // A nested call that ends with `exit` returns from that
                // invocation only; the current word keeps going.
                let flow = match flow {
                    Flow::Exit => Flow::Normal,
                    flow => flow,
                };

                Ok(Step {
                    flow,
                    advance: true,
                })
            }

            ":" => {
                let name = next_word(tokens, *pc + 1, ":")?;
                let body_start = *pc + 2;
                let body_end = tokens[body_start..]
                    .iter()
                    .position(|token| matches!(&token.kind, TokenKind::Word(word) if word == ";"))
                    .map(|offset| body_start + offset)
                    .ok_or_else(|| ForthError::new("missing ; in definition"))?;

                let body = tokens[body_start..body_end].to_vec();
                let compiled = CompiledWord::new(name.clone(), body)?;
                self.dictionary.define(&name, Rc::new(compiled));

                *pc = body_end;
                Ok(Step::normal())
            }

            ";" => Err(ForthError::new("unexpected ;")),

            "variable" => {
                let name = next_word(tokens, *pc + 1, "variable")?;

                if !matches!(self.dictionary.find(&name), Some(Word::Address(_))) {
                    let address = self.context.memory.allocate(1);
                    self.dictionary.define_address(&name, address);
                }

                *pc += 1;
                Ok(Step::normal())
            }

            "create" => {
                let name = next_word(tokens, *pc + 1, "create")?;
                let address = self.context.memory.here();
                self.dictionary.define_address(&name, address);

                *pc += 1;
                Ok(Step::normal())
            }

            "constant" => {
                let name = next_word(tokens, *pc + 1, "constant")?;
                let value = self.context.pop()?;
                self.dictionary.define_constant(&name, value);

                *pc += 1;
                Ok(Step::normal())
            }

            "value" => {
                let name = next_word(tokens, *pc + 1, "value")?;
                let value = self.context.pop()?;

                if let Some(Word::Value(address)) = self.dictionary.find(&name) {
                    self.context.memory.set(address as i64, value)?;
                } else {
                    let address = self.context.memory.push_cell(value);
                    self.dictionary.define_value(&name, address);
                }

                *pc += 1;
                Ok(Step::normal())
            }

            "to" => {
                let name = next_word(tokens, *pc + 1, "to")?;
                let value = self.context.pop()?;

                match self.dictionary.find(&name) {
                    Some(Word::Value(address)) | Some(Word::Address(address)) => {
                        self.context.memory.set(address as i64, value)?;
                    }
                    _ => {
                        return Err(ForthError::new(format!(
                            "to: {} is not a value or variable",
                            name
                        )));
                    }
                }

                *pc += 1;
                Ok(Step::normal())
            }

            "include" => {
                let path_token = tokens
                    .get(*pc + 1)
                    .ok_or_else(|| ForthError::new("expected path string after include"))?;
                let path = match &path_token.kind {
                    TokenKind::String(path) => path.clone(),
                    _ => return Err(ForthError::new("expected path string after include")),
                };

                self.include_file(&path, path_token)?;
                *pc += 1;
                Ok(Step::normal())
            }

            _ => {
                let flow = self.call_word(word, token)?;
                Ok(Step {
                    flow,
                    advance: true,
                })
            }
        }
    }

    fn call_word(&mut self, name: &str, token: &Token) -> Result<Flow, ForthError> {
        match self.dictionary.find(name) {
            Some(Word::Builtin(function)) => {
                function(&mut self.context)?;
                Ok(Flow::Normal)
            }
            Some(Word::UserDefined(word)) => {
                let flow =
                    self.execute_word_with_frame(name, word, Some(token.location.clone()))?;

                // `exit` inside a called word has already unwound that word's
                // own body; for the caller it is just a normal return.
                match flow {
                    Flow::Exit => Ok(Flow::Normal),
                    flow => Ok(flow),
                }
            }
            Some(Word::Color(color)) => {
                self.context.graphics.set_color(color);
                Ok(Flow::Normal)
            }
            Some(Word::Address(address)) => {
                self.context.push(address as i64);
                Ok(Flow::Normal)
            }
            Some(Word::Constant(value)) => {
                self.context.push(value);
                Ok(Flow::Normal)
            }
            Some(Word::Value(address)) => {
                let value = self.context.memory.get(address as i64)?;
                self.context.push(value);
                Ok(Flow::Normal)
            }
            None => Err(ForthError::new(format!("unknown word: {}", name))),
        }
    }

    fn call_named_word(
        &mut self,
        name: &str,
        location: Option<SourceLocation>,
    ) -> Result<Flow, ForthError> {
        match self.dictionary.find(name) {
            Some(Word::UserDefined(word)) => self.execute_word_with_frame(name, word, location),
            Some(Word::Builtin(function)) => {
                function(&mut self.context)?;
                Ok(Flow::Normal)
            }
            Some(Word::Color(color)) => {
                self.context.graphics.set_color(color);
                Ok(Flow::Normal)
            }
            Some(Word::Address(address)) => {
                self.context.push(address as i64);
                Ok(Flow::Normal)
            }
            Some(Word::Constant(value)) => {
                self.context.push(value);
                Ok(Flow::Normal)
            }
            Some(Word::Value(address)) => {
                let value = self.context.memory.get(address as i64)?;
                self.context.push(value);
                Ok(Flow::Normal)
            }
            None => Ok(Flow::Normal),
        }
    }

    fn execute_word_with_frame(
        &mut self,
        name: &str,
        word: Rc<CompiledWord>,
        location: Option<SourceLocation>,
    ) -> Result<Flow, ForthError> {
        self.call_stack.push(CallFrame::new(name, location));

        let result = self.execute_compiled(&word);
        let stack = self.call_stack.clone();
        self.call_stack.pop();

        match result {
            Ok(flow) => Ok(flow),
            Err(error) => Err(error.with_call_stack(stack)),
        }
    }

    fn include_file(&mut self, requested: &str, token: &Token) -> Result<(), ForthError> {
        let resolved = self.resolve_include_path(requested);

        let canonical = resolved.canonicalize().map_err(|error| {
            ForthError::at(
                format!("cannot include '{}': {}", resolved.display(), error),
                token.location.clone(),
            )
        })?;

        if self.include_stack.iter().any(|path| path == &canonical) {
            return Err(ForthError::at(
                format!("include cycle detected: {}", canonical.display()),
                token.location.clone(),
            ));
        }

        let source = std::fs::read_to_string(&canonical).map_err(|error| {
            ForthError::at(
                format!("cannot read '{}': {}", canonical.display(), error),
                token.location.clone(),
            )
        })?;

        let file_name = canonical.display().to_string();
        let tokens = tokenize(&source, &file_name)?;
        let compiled = CompiledWord::new(file_name, tokens)?;

        self.remember_file(&canonical);
        self.include_stack.push(canonical.clone());

        let result = self.execute_compiled(&compiled);
        self.include_stack.pop();

        match result? {
            Flow::Normal | Flow::Exit => Ok(()),
            Flow::Leave => Err(ForthError::at(
                "leave outside loop in included file",
                token.location.clone(),
            )),
        }
    }

    fn resolve_include_path(&self, requested: &str) -> PathBuf {
        let requested_path = PathBuf::from(requested);

        if requested_path.is_absolute() {
            return requested_path;
        }

        let base = self
            .include_stack
            .last()
            .and_then(|path| path.parent())
            .map(PathBuf::from)
            .or_else(|| {
                self.source_path
                    .as_ref()
                    .and_then(|path| path.parent())
                    .map(PathBuf::from)
            })
            .unwrap_or_else(|| PathBuf::from("."));

        base.join(requested_path)
    }
}

fn next_word(tokens: &[Token], index: usize, form: &str) -> Result<String, ForthError> {
    match tokens.get(index).and_then(Token::word) {
        Some(word) => Ok(word.to_string()),
        None => Err(ForthError::new(format!(
            "expected word name after {}",
            form
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::DrawCommand;

    fn eval(source: &str) -> Interpreter {
        let mut interpreter = Interpreter::new();
        interpreter.eval(source).unwrap();
        interpreter
    }

    #[test]
    fn arithmetic_and_stack_words() {
        let mut interpreter = eval("7 3 + 2 * 5 / 1 - dup");
        assert_eq!(interpreter.context.stack.pop(), Some(3));
        assert_eq!(interpreter.context.stack.pop(), Some(3));
    }

    #[test]
    fn variables_store_fetch_and_add() {
        let mut interpreter = eval("variable x 42 x ! 5 x +! x @");
        assert_eq!(interpreter.context.stack.pop(), Some(47));
    }

    #[test]
    fn variable_definitions_are_idempotent() {
        let mut interpreter = Interpreter::new();
        interpreter.eval("variable x 42 x !").unwrap();
        interpreter.eval("variable x x @").unwrap();
        assert_eq!(interpreter.context.stack.pop(), Some(42));
    }

    #[test]
    fn invalid_address_errors() {
        let mut interpreter = Interpreter::new();
        assert!(interpreter.eval("999 @").is_err());
    }

    #[test]
    fn constants_and_values() {
        let mut interpreter =
            eval("10 constant ten ten 5 + 1 value score score 2 + to score score");
        assert_eq!(interpreter.context.stack.pop(), Some(3));
        assert_eq!(interpreter.context.stack.pop(), Some(15));
    }

    #[test]
    fn create_allot_and_comma() {
        let mut interpreter =
            eval("create arr 5 cells allot 10 arr 0 + ! 20 arr 1 + ! arr 0 + @ arr 1 + @ +");
        assert_eq!(interpreter.context.stack.pop(), Some(30));

        let mut interpreter = eval("create data 42 , 7 , data @ data 1 + @ +");
        assert_eq!(interpreter.context.stack.pop(), Some(49));
    }

    #[test]
    fn do_loop_and_leave() {
        let mut interpreter =
            eval("variable sum 0 sum ! : add-all 10 0 do i sum +! loop ; add-all sum @");
        assert_eq!(interpreter.context.stack.pop(), Some(45));

        let mut interpreter = eval(
            "variable count 0 count ! : stop-early 10 0 do i 5 = if leave then 1 count +! loop ; stop-early count @",
        );
        assert_eq!(interpreter.context.stack.pop(), Some(5));
    }

    #[test]
    fn while_repeat_loop() {
        let mut interpreter =
            eval("variable n 0 n ! : up begin n @ 5 < while n @ 1 + n ! repeat ; up n @");
        assert_eq!(interpreter.context.stack.pop(), Some(5));
    }

    #[test]
    fn exit_and_recurse() {
        let mut interpreter = eval(": early 1 2 exit 3 + ; early");
        assert_eq!(interpreter.context.stack.pop(), Some(2));
        assert_eq!(interpreter.context.stack.pop(), Some(1));

        let mut interpreter = eval(": fact dup 1 > if dup 1 - recurse * else drop 1 then ; 5 fact");
        assert_eq!(interpreter.context.stack.pop(), Some(120));
    }

    #[test]
    fn exit_is_contained_to_the_called_word() {
        // `exit` inside a callee returns from that word only; tokens after
        // the call still execute and the top-level program continues.
        let mut interpreter = eval(": not-my-exit dup 0 = if exit then drop 99 ; 0 not-my-exit 42");
        assert_eq!(interpreter.context.stack.pop(), Some(42));
        assert_eq!(interpreter.context.stack.pop(), Some(0));

        // `exit` in the base case of a recursive word stays in the base case
        let mut interpreter =
            eval(": fib dup 2 < if exit then dup 1 - recurse swap 2 - recurse + ; 10 fib");
        assert_eq!(interpreter.context.stack.pop(), Some(55));
    }

    #[test]
    fn return_stack_words() {
        let mut interpreter = eval("10 >r 20 >r r@ r> r> + +");
        assert_eq!(interpreter.context.stack.pop(), Some(50));
    }

    #[test]
    fn fixed_point_math() {
        let mut interpreter = eval("1500 2000 f* 2500 500 f/ 500 fixed int");
        assert_eq!(interpreter.context.stack.pop(), Some(500));
        assert_eq!(interpreter.context.stack.pop(), Some(5000));
        assert_eq!(interpreter.context.stack.pop(), Some(3000));
    }

    #[test]
    fn strings_and_text_commands() {
        let mut interpreter = eval("\"hello\" string-length");
        assert_eq!(interpreter.context.stack.pop(), Some(5));

        let mut interpreter = Interpreter::new();
        interpreter
            .eval("\"hello\" 10 20 30 rl-white draw-text")
            .unwrap();

        match interpreter.context.graphics.commands.last().unwrap() {
            DrawCommand::Text {
                text, x, y, size, ..
            } => {
                assert_eq!(text, "hello");
                assert_eq!((*x, *y, *size), (10, 20, 30));
            }
            _ => panic!("expected a text command"),
        }
    }

    #[test]
    fn comments_are_ignored() {
        let mut interpreter = eval("\\ line comment\n( block comment ) 42");
        assert_eq!(interpreter.context.stack.pop(), Some(42));
    }

    #[test]
    fn lifecycle_calls_init_update_and_draw() {
        let mut interpreter = Interpreter::new();
        interpreter
            .load(
                "variable x : init 10 x ! ; : update 1 x +! ; : draw x @ 0 10 rl-white draw-circle ;",
                None,
            )
            .unwrap();

        assert_eq!(interpreter.context.stack.len(), 0);
        interpreter.run_frame().unwrap();

        match interpreter.context.graphics.commands.last().unwrap() {
            DrawCommand::Circle { x, .. } => assert_eq!(*x, 11),
            _ => panic!("expected a circle"),
        }

        interpreter.run_frame().unwrap();

        match interpreter.context.graphics.commands.last().unwrap() {
            DrawCommand::Circle { x, .. } => assert_eq!(*x, 12),
            _ => panic!("expected a circle"),
        }
    }

    #[test]
    fn diagnostics_include_location_and_call_stack() {
        let mut interpreter = Interpreter::new();
        let error = interpreter.eval(": bad draw-rectangel ; bad").unwrap_err();
        let message = error.to_string();

        assert!(message.contains("unknown word: draw-rectangel"));
        assert!(message.contains("<eval>:1:"));
        assert!(message.contains("in bad"));
    }

    #[test]
    fn include_resolves_relative_paths_and_detects_cycles() {
        let directory =
            std::env::temp_dir().join(format!("gameforth-include-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(directory.join("lib")).unwrap();

        std::fs::write(directory.join("lib/math.fth"), ": double 2 * ;\n").unwrap();
        std::fs::write(
            directory.join("lib/outer.fth"),
            "include \"math.fth\"\n: quadruple double double ;\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("main.fth"),
            "include \"lib/outer.fth\"\n21 quadruple\n",
        )
        .unwrap();

        let mut interpreter = Interpreter::new();
        interpreter.load_file(&directory.join("main.fth")).unwrap();
        assert_eq!(interpreter.context.stack.pop(), Some(84));

        std::fs::write(directory.join("cycle-a.fth"), "include \"cycle-b.fth\"\n").unwrap();
        std::fs::write(directory.join("cycle-b.fth"), "include \"cycle-a.fth\"\n").unwrap();

        let mut interpreter = Interpreter::new();
        let error = interpreter
            .load_file(&directory.join("cycle-a.fth"))
            .unwrap_err();
        assert!(error.to_string().contains("include cycle"));

        let _ = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn legacy_bounce_example_still_runs_with_lifecycle() {
        let source = std::fs::read_to_string("examples/test.fth").unwrap();
        let mut interpreter = Interpreter::new();
        interpreter
            .load(&source, Some(PathBuf::from("examples/test.fth")))
            .unwrap();

        for _ in 0..500 {
            interpreter.run_frame().unwrap();

            match interpreter.context.graphics.commands.last().unwrap() {
                DrawCommand::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    assert!((0..=700).contains(x));
                    assert!((0..=500).contains(y));
                    assert_eq!(*width, 100);
                    assert_eq!(*height, 100);
                }
                _ => panic!("expected a rectangle"),
            }
        }
    }
}
