pub struct TerminalParser;

impl TerminalParser {
    pub fn decode(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).to_string()
    }
}
