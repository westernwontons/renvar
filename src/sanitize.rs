/// Determines whether the input [`char`]
/// is a single quote ('), double quote (") or whitespace
pub(crate) fn is_quote_or_whitespace(c: char) -> bool {
    c == '"' || c == '\'' || c == ' '
}
