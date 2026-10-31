use std::collections::HashMap;

use crate::error::{ForthError, SourceLocation};
use crate::lexer::Token;

/// Precomputed jump targets for the structured control words in one token stream.
#[derive(Debug, Clone, Default)]
pub struct ControlMap {
    /// `if` -> matching `else`
    pub if_else: HashMap<usize, usize>,
    /// `if` -> matching `then`
    pub if_then: HashMap<usize, usize>,
    /// `else` -> matching `then`
    pub else_then: HashMap<usize, usize>,
    /// `begin` -> matching `until` or `repeat`
    pub begin_end: HashMap<usize, usize>,
    /// `while` -> matching `repeat`
    pub while_repeat: HashMap<usize, usize>,
    /// `repeat` -> matching `begin`
    pub repeat_begin: HashMap<usize, usize>,
    /// `until` -> matching `begin`
    pub until_begin: HashMap<usize, usize>,
    /// `do`/`?do` -> matching `loop`/`+loop`
    pub do_loop: HashMap<usize, usize>,
    /// `loop`/`+loop` -> matching `do`/`?do`
    pub loop_do: HashMap<usize, usize>,
}

/// A token stream plus its precomputed control-flow table.
#[derive(Debug, Clone)]
pub struct CompiledWord {
    pub name: String,
    pub tokens: Vec<Token>,
    pub control: ControlMap,
}

impl CompiledWord {
    pub fn new(name: impl Into<String>, tokens: Vec<Token>) -> Result<Self, ForthError> {
        let control = compile_control(&tokens)?;
        Ok(Self {
            name: name.into(),
            tokens,
            control,
        })
    }
}

#[derive(Debug, Clone)]
enum OpenKind {
    If {
        if_index: usize,
        else_index: Option<usize>,
    },
    Begin {
        begin_index: usize,
    },
    While {
        while_index: usize,
        begin_index: usize,
    },
    Do {
        do_index: usize,
    },
}

#[derive(Debug, Clone)]
struct OpenControl {
    kind: OpenKind,
    location: SourceLocation,
}

pub fn compile_control(tokens: &[Token]) -> Result<ControlMap, ForthError> {
    let mut map = ControlMap::default();
    let mut open: Vec<OpenControl> = Vec::new();

    let mut index = 0;

    while index < tokens.len() {
        let token = &tokens[index];

        let Some(word) = token.word() else {
            index += 1;
            continue;
        };

        if word == ":" {
            // A definition body is compiled separately. Skip it here so its
            // control words are not paired with control words outside it.
            let mut cursor = index + 1;
            if cursor < tokens.len() {
                cursor += 1; // skip the name
            }
            while cursor < tokens.len() && tokens[cursor].word() != Some(";") {
                cursor += 1;
            }
            index = cursor.saturating_add(1);
            continue;
        }

        match word {
            "if" => open.push(OpenControl {
                kind: OpenKind::If {
                    if_index: index,
                    else_index: None,
                },
                location: token.location.clone(),
            }),

            "else" => {
                let Some(top) = open.last_mut() else {
                    return Err(ForthError::at("else without if", token.location.clone()));
                };

                match &mut top.kind {
                    OpenKind::If { else_index, .. } => {
                        if else_index.is_some() {
                            return Err(ForthError::at("duplicate else", token.location.clone()));
                        }
                        *else_index = Some(index);
                    }
                    _ => return Err(ForthError::at("else without if", token.location.clone())),
                }
            }

            "then" => {
                let Some(top) = open.pop() else {
                    return Err(ForthError::at("then without if", token.location.clone()));
                };

                match top.kind {
                    OpenKind::If {
                        if_index,
                        else_index,
                    } => {
                        map.if_then.insert(if_index, index);
                        if let Some(else_index) = else_index {
                            map.if_else.insert(if_index, else_index);
                            map.else_then.insert(else_index, index);
                        }
                    }
                    _ => return Err(ForthError::at("then without if", token.location.clone())),
                }
            }

            "begin" => open.push(OpenControl {
                kind: OpenKind::Begin { begin_index: index },
                location: token.location.clone(),
            }),

            "while" => {
                let begin_index = match open.last() {
                    Some(OpenControl {
                        kind: OpenKind::Begin { begin_index },
                        ..
                    }) => *begin_index,
                    _ => {
                        return Err(ForthError::at(
                            "while without begin",
                            token.location.clone(),
                        ));
                    }
                };

                open.push(OpenControl {
                    kind: OpenKind::While {
                        while_index: index,
                        begin_index,
                    },
                    location: token.location.clone(),
                });
            }

            "repeat" => {
                let Some(while_control) = open.pop() else {
                    return Err(ForthError::at(
                        "repeat without while",
                        token.location.clone(),
                    ));
                };

                let OpenKind::While {
                    while_index,
                    begin_index,
                } = while_control.kind
                else {
                    return Err(ForthError::at(
                        "repeat without while",
                        token.location.clone(),
                    ));
                };

                let Some(begin_control) = open.pop() else {
                    return Err(ForthError::at(
                        "repeat without begin",
                        token.location.clone(),
                    ));
                };

                let OpenKind::Begin {
                    begin_index: begin_index_from_stack,
                } = begin_control.kind
                else {
                    return Err(ForthError::at(
                        "repeat without begin",
                        token.location.clone(),
                    ));
                };

                if begin_index != begin_index_from_stack {
                    return Err(ForthError::at(
                        "mismatched begin/repeat",
                        token.location.clone(),
                    ));
                }

                map.begin_end.insert(begin_index, index);
                map.while_repeat.insert(while_index, index);
                map.repeat_begin.insert(index, begin_index);
            }

            "until" => {
                let Some(begin_control) = open.pop() else {
                    return Err(ForthError::at(
                        "until without begin",
                        token.location.clone(),
                    ));
                };

                let OpenKind::Begin { begin_index } = begin_control.kind else {
                    return Err(ForthError::at(
                        "until without begin",
                        token.location.clone(),
                    ));
                };

                map.begin_end.insert(begin_index, index);
                map.until_begin.insert(index, begin_index);
            }

            "do" | "?do" => open.push(OpenControl {
                kind: OpenKind::Do { do_index: index },
                location: token.location.clone(),
            }),

            "loop" | "+loop" => {
                let Some(do_control) = open.pop() else {
                    return Err(ForthError::at("loop without do", token.location.clone()));
                };

                let OpenKind::Do { do_index } = do_control.kind else {
                    return Err(ForthError::at("loop without do", token.location.clone()));
                };

                map.do_loop.insert(do_index, index);
                map.loop_do.insert(index, do_index);
            }

            _ => {}
        }

        index += 1;
    }

    if let Some(unclosed) = open.last() {
        let message = match unclosed.kind {
            OpenKind::If { .. } => "missing then for if",
            OpenKind::Begin { .. } => "missing until or repeat for begin",
            OpenKind::While { .. } => "missing repeat for while",
            OpenKind::Do { .. } => "missing loop for do",
        };

        return Err(ForthError::at(message, unclosed.location.clone()));
    }

    Ok(map)
}
