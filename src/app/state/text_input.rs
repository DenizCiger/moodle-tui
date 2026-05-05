#[derive(Debug, Clone, Default)]
pub struct TextInputState {
    pub value: String,
    pub cursor: usize,
    pub mask: bool,
}

impl TextInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_mask(mut self, mask: bool) -> Self {
        self.mask = mask;
        self
    }

    pub fn set(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor = self.value.chars().count();
    }

    pub fn insert(&mut self, ch: char) {
        let byte_idx = self
            .value
            .char_indices()
            .nth(self.cursor)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len());
        self.value.insert(byte_idx, ch);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut chars: Vec<char> = self.value.chars().collect();
        chars.remove(self.cursor - 1);
        self.value = chars.into_iter().collect();
        self.cursor -= 1;
    }

    pub fn delete(&mut self) {
        let chars: Vec<char> = self.value.chars().collect();
        if self.cursor >= chars.len() {
            return;
        }
        let mut chars = chars;
        chars.remove(self.cursor);
        self.value = chars.into_iter().collect();
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.value.chars().count() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.value.chars().count();
    }

    pub fn display(&self) -> String {
        if self.mask {
            "*".repeat(self.value.chars().count())
        } else {
            self.value.clone()
        }
    }
}
