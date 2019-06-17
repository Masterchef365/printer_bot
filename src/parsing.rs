const BLOCKTEXT_PREFIX: &str = "```";
const BLOCKTEXT_POSTFIX: &str = "```";
const BLOCKTEXT_INVALIDATE: &str = "```";

const COMMAND_PREFIX: &str = "!print";

pub fn parse_command_body(content: &str) -> Option<&str> {
    if content.starts_with(COMMAND_PREFIX) {
        Some(
            content
                .trim_start_matches(COMMAND_PREFIX) //Remove command prefix
                .trim_start_matches(' ') //Remove trailing whitespace
                .trim_start_matches('\n'),
        )
    } else {
        None
    }
}

pub fn parse_blocktext(content: &str) -> Option<&str> {
    if content.starts_with(BLOCKTEXT_PREFIX) && content.ends_with(BLOCKTEXT_POSTFIX) {
        let trimmed = content
            .trim_start_matches(BLOCKTEXT_PREFIX)
            .trim_end_matches(BLOCKTEXT_POSTFIX);
        if !trimmed.contains(BLOCKTEXT_INVALIDATE) {
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
    use super::{parse_blocktext, parse_command_body};
    #[test]
    fn test_parse_message() {
        assert_eq!(parse_blocktext("invalid"), None);
        assert_eq!(parse_blocktext("```valid```"), Some("valid"));
        assert_eq!(parse_blocktext("```valid```"), Some("valid"));
        assert_eq!(parse_blocktext("```fdalsjfd```invalid```"), None);
    }

    #[test]
    fn test_parse_command_body() {
        assert_eq!(parse_command_body("jfdal; dasj fldjeio afd af;l"), None);
        assert_eq!(parse_command_body("fdal; https://google.com"), None);
        assert_eq!(parse_command_body("!print"), Some(""));
        assert_eq!(
            parse_command_body("!print   https://google.com"),
            Some("https://google.com")
        );
    }
}
