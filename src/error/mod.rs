pub(crate) mod object_error;
pub(crate) mod parse_error;
pub(crate) mod resolve_error;
pub(crate) mod runtime_error;
pub(crate) mod syntax_error;

pub(crate) trait ErrorReporter<E>
where
    E: ToString,
{
    fn errors(&self) -> &[E];

    fn had_error(&self) -> bool {
        !self.errors().is_empty()
    }

    fn error_string(&self) -> String {
        self.errors()
            .iter()
            .map(|err| err.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
