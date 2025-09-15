#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TerminalWidgetCurrentLine {
    /// User's input.
    Input(String),

    /// Some running command's output.
    Output(String)
}

impl Default for TerminalWidgetCurrentLine {
    #[inline]
    fn default() -> Self {
        Self::Input(String::new())
    }
}

// TODO: inline terminal hints

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TerminalWidget {
    pub history: Vec<String>,
    pub ongoing: TerminalWidgetCurrentLine,
    pub prefix: Option<String>,
    pub offset: Option<usize>,
    pub height: u16
}

impl TerminalWidget {
    pub fn prefix(&self, text: impl AsRef<str>) -> String {
        match &self.prefix {
            Some(prefix) => format!("{prefix} > {}", text.as_ref()),
            None => format!("> {}", text.as_ref())
        }
    }

    pub fn push(&mut self, text: impl AsRef<str>) {
        for line in text.as_ref().lines() {
            self.history.push(line.to_string());
        }
    }

    pub fn allow_user_input(&mut self) -> TerminalWidgetCurrentLine {
        let prev = self.ongoing.clone();

        self.ongoing = TerminalWidgetCurrentLine::Input(String::new());

        prev
    }

    pub fn forbid_user_input(&mut self) -> TerminalWidgetCurrentLine {
        let prev = self.ongoing.clone();

        self.ongoing = TerminalWidgetCurrentLine::Output(String::new());

        prev
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn stick_offset(&self, height: usize) -> usize {
        let input_height = match &self.ongoing {
            TerminalWidgetCurrentLine::Input(text) |
            TerminalWidgetCurrentLine::Output(text) => {
                text.lines()
                    .count()
                    .max(1)
            }
        };

        let lines = self.len();

        if lines + input_height > height {
            (lines + input_height).saturating_sub(height)
        } else {
            0
        }
    }

    pub fn lines(&self, offset: usize) -> Vec<String> {
        let mut lines = self.history.iter()
            .skip(offset)
            .cloned()
            .collect::<Vec<String>>();

        match &self.ongoing {
            TerminalWidgetCurrentLine::Input(text) => {
                lines.push(self.prefix(text));
            }

            TerminalWidgetCurrentLine::Output(text) => {
                lines.push(text.to_string());
            }
        }

        lines
    }
}
