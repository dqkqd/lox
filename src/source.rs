use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, PartialEq, Hash, Copy, Default)]
pub(crate) struct CharPos {
    pub ch: char,
    pub index: usize,
    pub line: usize,
    pub width: usize,
}

#[derive(Debug)]
pub(crate) struct SourcePos {
    positions: Vec<CharPos>,
    newlines_pos: Vec<usize>,
}

impl SourcePos {
    pub fn new(source: &str) -> Self {
        let mut positions = Vec::with_capacity(source.len());

        let mut line = 0;

        let mut newlines_pos = vec![0];

        for (index, ch) in source.chars().enumerate() {
            let width = UnicodeWidthChar::width(ch).unwrap_or(0);

            let char_pos = CharPos {
                ch,
                index,
                line,
                width,
            };

            if ch == '\n' {
                line += 1;
                newlines_pos.push(index + 1);
            }

            positions.push(char_pos);
        }

        newlines_pos.push(positions.len());

        Self {
            positions,
            newlines_pos,
        }
    }

    pub fn get(&self, index: usize) -> Option<CharPos> {
        self.positions.get(index).cloned()
    }

    pub fn newlines_pos(&self) -> &[usize] {
        &self.newlines_pos
    }
}
