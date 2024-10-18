use clap::{Arg, ArgAction, Command};
use std::fs::File;
use std::io::{self, BufRead, Write};

const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

fn sort_by_numeric(lines: &mut Vec<String>, column: Option<usize>) {
    lines.sort_unstable_by(|a, b| {
        let a = get_column_value(a, column);
        let b = get_column_value(b, column);
        let num_a: f64 = a.parse().unwrap_or(f64::MIN);
        let num_b: f64 = b.parse().unwrap_or(f64::MIN);
        num_a.partial_cmp(&num_b).unwrap()
    });
}

fn sort_by_month(lines: &mut Vec<String>, column: Option<usize>) {
    lines.sort_unstable_by_key(|line| {
        let value = get_column_value(line, column);
        MONTHS.iter().position(|&month| value.contains(month)).unwrap_or(13)
    });
}

fn parse_with_suffix(s: &String) -> f64 {
    let len = s.len();
    if len == 0 {
        return f64::MIN;
    }

    if len == 1 {
        return s.parse().unwrap_or(f64::MIN);
    }
    let number_part: f64 = s[..len-1].parse().unwrap_or(0.0);
    let suffix = &s[len-1..];
    match suffix {
        "K"|"k" => number_part * 1_000.0,
        "M"|"m" => number_part * 1_000_000.0,
        "G"|"g" => number_part * 1_000_000_000.0,
        _ => number_part,
    }
}

fn sort_by_suffix(lines: &mut Vec<String>, column: Option<usize>) {
    lines.sort_unstable_by(|a, b| {
        let a = get_column_value(a, column);
        let b = get_column_value(b, column);
        let num_a = parse_with_suffix(&a);
        let num_b = parse_with_suffix(&b);
        num_a.partial_cmp(&num_b).unwrap()
    });
}

fn sort_by_string(lines: &mut Vec<String>, column: Option<usize>) {
    lines.sort_unstable_by_key(|line| get_column_value(line, column));
}

fn check_sorted_by_numeric(lines: &Vec<String>, column: Option<usize>, reversed: bool) -> bool {
    for i in 1..lines.len() {
        let a = get_column_value(&lines[i], column);
        let b = get_column_value(&lines[i-1], column);
        let num_a: f64 = a.parse().unwrap_or(f64::MIN);
        let num_b: f64 = b.parse().unwrap_or(f64::MIN);

        if !reversed {
            if num_a < num_b {
                return false;
            }
        }
        else {
            if num_b < num_a {
                return false;
            }
        }
    }
    true
}

fn check_sorted_by_month(lines: &Vec<String>, column: Option<usize>, reversed: bool) -> bool {
    for i in 1..lines.len() {
        let a = get_column_value(&lines[i], column);
        let b = get_column_value(&lines[i-1], column);
        let month_pos_a: usize = MONTHS.iter().position(|&month| a.contains(month)).unwrap_or(13);
        let month_pos_b: usize = MONTHS.iter().position(|&month| b.contains(month)).unwrap_or(13);

        if !reversed {
            if month_pos_a < month_pos_b {
                return false;
            }
        }
        else {
            if month_pos_b < month_pos_a {
                return false;
            }
        }
    }
    true
}

fn check_sorted_by_suffix(lines: &Vec<String>, column: Option<usize>, reversed: bool) -> bool {
    for i in 1..lines.len() {
        let a = get_column_value(&lines[i], column);
        let b = get_column_value(&lines[i-1], column);
        let num_a: f64 = parse_with_suffix(&a);
        let num_b: f64 = parse_with_suffix(&b);

        if !reversed {
            if num_a < num_b {
                return false;
            }
        }
        else {
            if num_b < num_a {
                return false;
            }
        }
    }
    true
}

fn get_column_value(line: &str, column: Option<usize>) -> String {
    column
        .and_then(|col| line.split_whitespace().nth(col - 1))
        .unwrap_or(line)
        .to_string()
}

fn main() -> io::Result<()> {
    let matches = Command::new("sort")
        .disable_help_flag(true)
        .arg(Arg::new("filename")
            .required(true)
            .index(1))

        .arg(Arg::new("k")
            .short('k')
            .default_value("1")
            .num_args(1))

        .arg(Arg::new("n")
            .short('n')
            .action(ArgAction::SetTrue)
            .conflicts_with_all(&["M", "h"]))

        .arg(Arg::new("r")
            .short('r')
            .action(ArgAction::SetTrue))

        .arg(Arg::new("u")
            .short('u')
            .action(ArgAction::SetTrue))

        .arg(Arg::new("M")
            .short('M')
            .action(ArgAction::SetTrue)
            .conflicts_with("h"))

        .arg(Arg::new("b")
            .short('b')
            .action(ArgAction::SetTrue))

        .arg(Arg::new("c")
            .short('c')
            .action(ArgAction::SetTrue)
            .requires_all(&["M", "h", "n"]))

        .arg(Arg::new("h")
            .short('h')
            .action(ArgAction::SetTrue))
        .get_matches();

    let filename = matches.get_one::<String>("filename").unwrap();
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    let reverse = matches.get_flag("r");
    let unique = matches.get_flag("u");
    let numeric = matches.get_flag("n");
    let month = matches.get_flag("M");
    let suffix = matches.get_flag("h");
    let check_sorted = matches.get_flag("c");
    let ignore_spaces = matches.get_flag("b");
    let column = matches.get_one::<String>("k").and_then(|c| c.parse::<usize>().ok());

    if ignore_spaces {
        lines = lines.into_iter().map(|line| line.trim_end().to_string()).collect();
    }

    if check_sorted {
        let mut sorted = true;

        if numeric {
            sorted = check_sorted_by_numeric(&lines, column, reverse);
        }

        if month {
            sorted = check_sorted_by_month(&lines, column, reverse);
        }

        if suffix {
            sorted = check_sorted_by_suffix(&lines, column, reverse);
        }

        if sorted {
            println!("Lines are sorted.");
        } else {
            println!("Lines are not sorted.");
        }
        return Ok(());
    }

    if numeric {
        sort_by_numeric(&mut lines, column);
    } else if month {
        sort_by_month(&mut lines, column);
    } else if suffix {
        sort_by_suffix(&mut lines, column);
    } else {
        sort_by_string(&mut lines, column);
    }

    if unique {
        lines.dedup();
    }

    if reverse {
        lines.reverse();
    }

    let sorted_filename = "sorted_".to_string() + &filename;
    File::create(sorted_filename)?.write_all(&lines.join("\n").as_bytes())?;
    Ok(())
}

