use crate::source::{CharPos, SourcePos};

const ERROR_MARK: char = '^';
const NORMAL_MARK: char = ' ';

fn string_equal_width(ch: char, width: usize) -> String {
    // endline character has width = 0, but we still need to indicate there is error
    let width = std::cmp::max(1, width);
    vec![ch; width].into_iter().collect::<String>()
}

fn error_string(pos: &CharPos) -> String {
    string_equal_width(ERROR_MARK, pos.width)
}

fn normal_string(pos: &CharPos) -> String {
    string_equal_width(NORMAL_MARK, pos.width)
}

pub(crate) trait ErrorPos: std::error::Error {
    fn start_pos(&self) -> CharPos;
    fn end_pos(&self) -> CharPos;
}

macro_rules! impl_error_pos {
    ($struct:ident) => {
        impl $crate::error::reporter::ErrorPos for $struct {
            fn start_pos(&self) -> $crate::source::CharPos {
                self.start_pos
            }

            fn end_pos(&self) -> $crate::source::CharPos {
                self.end_pos
            }
        }
    };
}

pub(crate) use impl_error_pos;

pub(crate) trait ErrorReporter<E>
where
    E: ErrorPos,
{
    fn errors(&self) -> &[E];

    fn had_error(&self) -> bool {
        !self.errors().is_empty()
    }

    fn error_msg(&self, reporter: &Reporter) -> String {
        self.errors()
            .iter()
            .map(|err| reporter.report(err))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub(crate) struct Reporter<'a> {
    source: &'a SourcePos,
}

impl<'a> Reporter<'a> {
    pub fn new(source: &'a SourcePos) -> Self {
        Self { source }
    }

    fn draw_one_line_error(
        &self,
        line_span_index: (usize, usize),
        error_span_index: (usize, usize),
    ) -> String {
        let mut src_string = String::new();
        let mut err_string = String::new();

        let is_error = |index: usize, error_span_index: (usize, usize)| -> bool {
            index >= error_span_index.0 && index <= error_span_index.1
        };

        for index in line_span_index.0..line_span_index.1 + 1 {
            let char_pos = self
                .source
                .get(index)
                .expect("Index never exceed source's range");
            src_string.push(char_pos.ch);

            if is_error(char_pos.index, error_span_index) {
                err_string.push_str(&error_string(&char_pos))
            } else {
                err_string.push_str(&normal_string(&char_pos))
            }
        }

        format!("{}\n{}", src_string.trim_end(), err_string.trim_end())
    }

    fn error_in_middle(&self, line: usize, start_pos: usize, end_pos: usize) -> String {
        let newlines_pos = self.source.newlines_pos();

        let start_line_index = newlines_pos[line];
        let end_line_index = newlines_pos[line + 1] - 1;

        self.draw_one_line_error((start_line_index, end_line_index), (start_pos, end_pos))
    }

    fn error_to_end(&self, line: usize, start_pos: usize) -> String {
        let newlines_pos = self.source.newlines_pos();

        let start_line_index = newlines_pos[line];
        let end_line_index = newlines_pos[line + 1] - 1;

        self.draw_one_line_error(
            (start_line_index, end_line_index),
            (start_pos, end_line_index),
        )
    }

    fn error_from_start(&self, line: usize, end_pos: usize) -> String {
        let newlines_pos = self.source.newlines_pos();

        let start_line_index = newlines_pos[line];
        let end_line_index = newlines_pos[line + 1] - 1;

        self.draw_one_line_error(
            (start_line_index, end_line_index),
            (start_line_index, end_pos),
        )
    }

    pub fn report<E>(&self, error: &E) -> String
    where
        E: ErrorPos,
    {
        let start_pos = error.start_pos();
        let end_pos = error.end_pos();

        let mut result = String::new();
        result.push_str(&format!("{}\n", error.to_string()));

        if start_pos.line == end_pos.line {
            // handle one line
            result.push_str(&self.error_in_middle(start_pos.line, start_pos.index, end_pos.index));
        } else {
            // handle multiple line

            // first line
            result.push_str(&self.error_to_end(start_pos.line, start_pos.index));

            // middle lines
            // end_pos.line - 1 will never be overflow since end_pos.line > start_pos.line >= 0
            for line in start_pos.line + 1..end_pos.line - 1 {
                result.push_str(&self.error_to_end(line, 0));
            }

            // last line
            result.push_str(&self.error_from_start(end_pos.line, end_pos.index));

            todo!("Add test for this case");
        }

        result
    }
}
