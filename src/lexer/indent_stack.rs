/// Tracks indentation levels for Python-style significant whitespace
#[derive(Debug, Default)]
pub struct IndentStack {
    levels: Vec<usize>,
}

impl IndentStack {
    pub fn new() -> Self {
        Self { levels: vec![0] }
    }

    /// Get the current indentation level
    pub fn current(&self) -> usize {
        *self.levels.last().unwrap_or(&0)
    }

    /// Process a new line's indentation, returning tokens to emit
    /// Returns: None if same level, Some(Indent) if increased, Some(Dedent) if decreased
    pub fn process_indent(&mut self, indent: usize) -> IndentChange {
        let current = self.current();

        if indent > current {
            self.levels.push(indent);
            IndentChange::Indent
        } else if indent < current {
            // Count how many levels we need to pop
            let mut dedent_count = 0;
            
            // Pop levels until we find matching indent
            while self.levels.len() > 1 && *self.levels.last().unwrap() > indent {
                self.levels.pop();
                dedent_count += 1;
            }

            // Check for inconsistent indentation
            if self.current() != indent {
                return IndentChange::Error(format!(
                    "Inconsistent indentation: expected {}, got {}",
                    self.current(),
                    indent
                ));
            }

            // Return the number of dedents needed
            if dedent_count > 0 {
                IndentChange::DedentCount(dedent_count)
            } else {
                IndentChange::Dedent
            }
        } else {
            IndentChange::None
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        self.levels = vec![0];
    }

    /// Get the depth of indentation
    pub fn depth(&self) -> usize {
        self.levels.len() - 1
    }
}

#[derive(Debug, PartialEq)]
pub enum IndentChange {
    None,
    Indent,
    Dedent,
    DedentCount(usize),
    Error(String),
}
