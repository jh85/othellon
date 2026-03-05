pub struct GtpCommand {
    pub id: Option<u32>,
    pub name: String,
    pub args: Vec<String>,
}

pub fn parse_line(line: &str) -> Option<GtpCommand> {
    // Strip comments
    let line = if let Some(idx) = line.find('#') {
        &line[..idx]
    } else {
        line
    };

    // Replace tabs with spaces, strip control chars except newline
    let line: String = line
        .chars()
        .map(|c| if c == '\t' { ' ' } else { c })
        .filter(|c| !c.is_control() || *c == '\n')
        .collect();

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let (id, cmd_start) = if let Ok(n) = parts[0].parse::<u32>() {
        (Some(n), 1)
    } else {
        (None, 0)
    };

    if cmd_start >= parts.len() {
        return None;
    }

    let name = parts[cmd_start].to_lowercase();
    let args = parts[cmd_start + 1..]
        .iter()
        .map(|s| s.to_string())
        .collect();

    Some(GtpCommand { id, name, args })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let cmd = parse_line("name").unwrap();
        assert_eq!(cmd.name, "name");
        assert!(cmd.id.is_none());
        assert!(cmd.args.is_empty());
    }

    #[test]
    fn test_command_with_id() {
        let cmd = parse_line("1 boardsize 8").unwrap();
        assert_eq!(cmd.id, Some(1));
        assert_eq!(cmd.name, "boardsize");
        assert_eq!(cmd.args, vec!["8"]);
    }

    #[test]
    fn test_comment_stripping() {
        let cmd = parse_line("name # this is a comment").unwrap();
        assert_eq!(cmd.name, "name");
    }

    #[test]
    fn test_empty_line() {
        assert!(parse_line("").is_none());
        assert!(parse_line("# comment only").is_none());
    }
}
