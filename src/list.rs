//! `aliases list`: aliases grouped by file, laid out in columns like `ls`.

use crate::ui::{DIM, GREEN, RESET, YELLOW};

pub struct Style {
    pub width: usize,
    pub color: bool,
    pub unicode: bool,
}

pub struct Group<'a> {
    pub file_name: &'a str,
    pub aliases: Vec<(&'a str, &'a str)>,
}

const INDENT: &str = "  ";
const GUTTER: &str = "    ";
const MIN_COMMAND_WIDTH: usize = 20;

fn len(text: &str) -> usize {
    text.chars().count()
}

/// `text` truncated with an ellipsis, or padded with spaces, to exactly `width` chars.
fn fit(text: &str, width: usize, ellipsis: &str) -> String {
    let mut out: String = if len(text) > width {
        let keep = width.saturating_sub(len(ellipsis));
        if keep > 0 {
            text.chars().take(keep).chain(ellipsis.chars()).collect()
        } else {
            text.chars().take(width).collect()
        }
    } else {
        text.to_string()
    };
    let pad = width.saturating_sub(len(&out));
    out.extend(std::iter::repeat_n(' ', pad));
    out
}

/// Width of each cell when a line holds `columns` cells.
fn cell_width(width: usize, columns: usize) -> usize {
    width.saturating_sub(INDENT.len() + GUTTER.len() * (columns - 1)) / columns
}

/// Renders the groups; empty groups are skipped. Returns `None` when there is nothing to show.
pub fn render(groups: &[Group], style: &Style) -> Option<String> {
    let paint = |color: &str, text: &str| {
        if style.color {
            format!("{color}{text}{RESET}")
        } else {
            text.to_string()
        }
    };
    let (ellipsis, rule_char) = if style.unicode {
        ("…", '─')
    } else {
        ("...", '-')
    };
    let max_columns = match style.width {
        w if w >= 120 => 3,
        w if w >= 70 => 2,
        _ => 1,
    };

    let mut out = String::new();
    let (mut total_aliases, mut total_files) = (0, 0);

    for group in groups.iter().filter(|g| !g.aliases.is_empty()) {
        let count = group.aliases.len();
        if total_files > 0 {
            out.push('\n');
        }

        // Header: "docker.alias ──────────── 6"
        let label = count.to_string();
        let rule: String = std::iter::repeat_n(
            rule_char,
            style
                .width
                .saturating_sub(len(group.file_name) + label.len() + 2),
        )
        .collect();
        out.push_str(&paint(YELLOW, group.file_name));
        out.push(' ');
        out.push_str(&paint(DIM, &format!("{rule} {label}")));
        out.push('\n');

        // Names are never cut, so drop a column while the longest name leaves no room
        // for a usable command.
        let mut name_width = group.aliases.iter().map(|(n, _)| len(n)).max().unwrap_or(0);
        let mut columns = count.min(max_columns);
        let (rows, cell) = loop {
            let rows = count.div_ceil(columns);
            columns = count.div_ceil(rows);
            let cell = cell_width(style.width, columns);
            if columns == 1 || name_width + 2 + MIN_COMMAND_WIDTH <= cell {
                break (rows, cell);
            }
            columns -= 1;
        };
        // Only a very narrow terminal can still leave no room; then names share the cell.
        if name_width + 2 + MIN_COMMAND_WIDTH > cell {
            name_width = cell / 2;
        }
        let command_width = cell.saturating_sub(name_width + 2).max(1);

        // Fill column by column, like ls.
        for row in 0..rows {
            let mut line = INDENT.to_string();
            for column in 0..columns {
                let Some((name, command)) = group.aliases.get(column * rows + row) else {
                    break;
                };
                if column > 0 {
                    line.push_str(GUTTER);
                }
                line.push_str(&paint(GREEN, &fit(name, name_width, ellipsis)));
                line.push_str("  ");
                line.push_str(&fit(command, command_width, ellipsis));
            }
            out.push_str(line.trim_end_matches(' '));
            out.push('\n');
        }

        total_aliases += count;
        total_files += 1;
    }

    if total_files == 0 {
        return None;
    }
    let plural =
        |n: usize, word: &str, words: &str| format!("{n} {}", if n == 1 { word } else { words });
    out.push('\n');
    out.push_str(&paint(
        DIM,
        &format!(
            "{} in {}",
            plural(total_aliases, "alias", "aliases"),
            plural(total_files, "file", "files")
        ),
    ));
    out.push('\n');
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style(width: usize) -> Style {
        Style {
            width,
            color: false,
            unicode: true,
        }
    }

    #[test]
    fn lays_out_in_columns_filled_top_to_bottom() {
        let groups = [Group {
            file_name: "git.alias",
            aliases: vec![
                ("ga", "git add"),
                ("gc", "git commit"),
                ("gs", "git status"),
            ],
        }];
        let out = render(&groups, &style(80)).unwrap();
        let lines: Vec<_> = out.lines().collect();
        assert_eq!(
            lines[0],
            format!("git.alias {} 3", "─".repeat(80 - 9 - 1 - 2))
        );
        assert!(lines[1].starts_with("  ga  git add"));
        assert!(lines[1].contains("    gs  git status"));
        assert_eq!(lines[2], "  gc  git commit");
        assert_eq!(lines.last().unwrap(), &"3 aliases in 1 file");
    }

    #[test]
    fn truncates_long_commands_but_not_names() {
        let long = "x".repeat(100);
        let groups = [Group {
            file_name: "a.alias",
            aliases: vec![("name", long.as_str())],
        }];
        let out = render(&groups, &style(40)).unwrap();
        let line = out.lines().nth(1).unwrap();
        assert_eq!(line.chars().count(), 40);
        assert!(line.starts_with("  name  xxx"));
        assert!(line.ends_with('…'));
    }

    #[test]
    fn uses_ascii_without_unicode() {
        let long = "y".repeat(100);
        let groups = [Group {
            file_name: "a.alias",
            aliases: vec![("n", long.as_str())],
        }];
        let out = render(
            &groups,
            &Style {
                width: 30,
                color: false,
                unicode: false,
            },
        )
        .unwrap();
        assert!(out.lines().next().unwrap().contains("---"));
        assert!(out.lines().nth(1).unwrap().ends_with("..."));
    }

    #[test]
    fn nothing_to_show() {
        let groups = [Group {
            file_name: "a.alias",
            aliases: vec![],
        }];
        assert!(render(&groups, &style(80)).is_none());
    }
}
