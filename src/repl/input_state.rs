use crate::lexer::indent_stack::IndentStack;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LineStatus {
    Complete,   // Ready to compile and run
    Incomplete, // Inside a multi-line block, need more input
    Empty,      // Empty line (often used to close a multi-line block)
}

pub struct InputState {
    buffer: String,
    bracket_depth: i32,
    inside_block: bool,
    indent_stack: IndentStack,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            bracket_depth: 0,
            inside_block: false,
            indent_stack: IndentStack::new(),
        }
    }

    /// Feeds a new line of input into the state machine.
    pub fn feed(&mut self, line: &str) -> LineStatus {
        let trimmed = line.trim();

        // Check for empty line
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return if self.inside_block || self.indent_stack.depth() > 0 {
                LineStatus::Empty
            } else {
                LineStatus::Complete
            };
        }

        // Add line to buffer
        self.buffer.push_str(line);
        self.buffer.push('\n');

        // Check brackets using a simple scan (ignoring strings for now, can be improved)
        let mut in_string = false;
        let mut string_char = '\0';
        let mut string_escaped = false;

        for c in line.chars() {
            if in_string {
                if string_escaped {
                    string_escaped = false;
                } else if c == '\\' {
                    string_escaped = true;
                } else if c == string_char {
                    in_string = false;
                }
            } else {
                match c {
                    '\'' | '"' => {
                        in_string = true;
                        string_char = c;
                    }
                    '(' | '[' | '{' => self.bracket_depth += 1,
                    ')' | ']' | '}' => self.bracket_depth -= 1,
                    '#' => break, // Ignore rest of line
                    _ => {}
                }
            }
        }

        // Count indentation
        let mut indent = 0;
        for c in line.chars() {
            if c == ' ' {
                indent += 1;
            } else if c == '\t' {
                indent = (indent / 4 + 1) * 4;
            } else {
                break;
            }
        }

        let _ = self.indent_stack.process_indent(indent);

        // Check if inside block (ends with :)
        // We strip comments to check for the trailing colon
        let code_part = if let Some(idx) = line.find('#') { &line[..idx] } else { line };

        if code_part.trim_end().ends_with(':') {
            self.inside_block = true;
        }

        // Empty line at indent > 0 or after block resolves block
        // This is handled by the caller manually calling complete_block

        if self.bracket_depth > 0 || self.inside_block || self.indent_stack.depth() > 0 {
            LineStatus::Incomplete
        } else {
            LineStatus::Complete
        }
    }

    /// Forcefully completes the current block
    pub fn force_complete(&mut self) {
        self.inside_block = false;
        self.indent_stack.reset();
        self.bracket_depth = 0;
    }

    /// Takes the accumulated buffer and resets the state
    pub fn take_buffer(&mut self) -> String {
        let res = std::mem::take(&mut self.buffer);
        self.force_complete();
        res
    }
}
