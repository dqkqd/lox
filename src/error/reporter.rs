use crate::source::SourcePos;

use super::syntax_error::SyntaxError;

pub(crate) struct Reporter<'a> {
    source: &'a SourcePos,
}

impl<'a> Reporter<'a> {
    pub fn new(source: &'a SourcePos) -> Self {
        Self { source }
    }

    pub fn report(&self, error: &SyntaxError) -> String {
        let start_pos = error.start_pos();
        let end_pos = error.end_pos();

        let newlines_pos = self.source.newlines_pos();

        // handle one line
        if start_pos.line == end_pos.line {
            let mut src = String::new();
            let mut err = String::new();
            for index in newlines_pos[start_pos.line]..newlines_pos[start_pos.line + 1] {
                if let Some(char_pos) = self.source.get(index) {
                    src.push(char_pos.ch);
                    if char_pos.index >= start_pos.index && char_pos.index <= end_pos.index {
                        err.push('^');
                    } else {
                        err.push(' ');
                    }
                }
            }
            let src = src.trim_end();
            let err = err.trim_end();
            format!("{}\n{}", src, err)
        } else {
            todo!()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{error::ErrorReporter, scanner::Scanner};

    use super::*;

    use std::io::Write;

    fn test_scanner(source: &str, expected_output: &str) -> Result<(), std::io::Error> {
        let mut result = Vec::new();
        let mut scanner = Scanner::new(source);

        scanner.scan_tokens();

        let source_pos = SourcePos::new(source);
        let reporter = Reporter::new(&source_pos);

        let errors = scanner
            .errors()
            .iter()
            .map(|err| reporter.report(err))
            .collect::<Vec<_>>()
            .join("\n");

        writeln!(&mut result, "{}", errors)?;

        let result = String::from_utf8(result).unwrap();
        assert_eq!(result.trim(), expected_output.trim());
        Ok(())
    }

    #[test]
    fn it_works() -> Result<(), std::io::Error> {
        let source = "!@#";
        let expected_output = r#"
!@#
 ^
!@#
  ^
"#;
        test_scanner(source, expected_output)
    }
}
