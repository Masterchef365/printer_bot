const COMMAND_PREFIX: &str = "```";
const COMMAND_POSTFIX: &str = "```";
const COMMAND_INVALIDATE: &str = "```";

pub fn parse_message(content: &str) -> Option<&str> {
    if content.starts_with(COMMAND_PREFIX) && content.ends_with(COMMAND_POSTFIX) {
        let trimmed = content
            .trim_start_matches(COMMAND_PREFIX)
            .trim_end_matches(COMMAND_POSTFIX);
        if !trimmed.contains(COMMAND_INVALIDATE) {
            Some(trimmed)
        } else {
            None
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::parse_message;
    #[test]
    fn test_parse_message() {
        assert_eq!(parse_message("invalid"), None);
        assert_eq!(parse_message("```printervalid```"), Some("valid"));
        assert_eq!(parse_message("```printervalid```"), Some("valid"));
        assert_eq!(parse_message("```printer```invalid```"), None);
    }
}

