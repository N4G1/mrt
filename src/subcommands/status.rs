use super::super::argparse::ParsedArgs;
use super::super::config::configmodels::ConfigFile;
use super::super::execute;
use clap::ArgMatches;
use colored::Colorize;
use std::process::Command;

pub fn status(args: &ArgMatches, parsed_arguments: &ParsedArgs, config: ConfigFile) {
    println!("These are your tags {:#?}", parsed_arguments.tags);

    let paths = execute::get_all_paths(&parsed_arguments.tags, &config);

    for path in paths {
        println!("{}", run_status(&path));
    }
}

fn run_status(path: &String) -> String {
    let mut cmd = Command::new("git");

    cmd.args(&["-c", "color.ui=always"])
        .args(&["status", "--branch", "--porcelain"])
        .current_dir(path);

    match cmd.output() {
        Ok(output) => format_output(path, &output.stdout),
        _ => String::from(""),
    }
}

fn format_output(path: &String, out: &Vec<u8>) -> String {
    let x = String::from_utf8_lossy(out).to_string();
    let y: Vec<String> = x.split('\n').map(String::from).collect();
    let branch: String = get_branch(&y).unwrap_or(String::from("<UNKNOWN>"));
    let behindness: String = get_behindness(&y)
        .map(|b| format!(" {}", b.yellow()))
        .unwrap_or(String::new());

    let path_spaces_to_add = 50 - path.len();
    let path_spaces = " ".repeat(path_spaces_to_add);

    format!("{}{}{}{}", path, path_spaces, branch, behindness) // TODO: Expand with dirtyness
}

fn get_branch(lines: &Vec<String>) -> Option<String> {
    lines.first().map(|branch_line| {
        let mut split: Vec<String> = branch_line.split("## ").map(String::from).collect();
        if split.len() > 1 {
            split.reverse();
            split.pop();
            split.reverse();
        }
        let joined: String = split.join("## ");

        let mut dotsplit: Vec<String> = joined.split("...").map(String::from).collect();
        let middle_idx = dotsplit.len() / 2;
        while dotsplit.len() > middle_idx {
            dotsplit.pop();
        }
        dotsplit.join("...")
    })
}

fn get_behindness(lines: &Vec<String>) -> Option<String> {
    lines
        .first()
        .map(|branch_line| {
            if branch_line.ends_with("]") {
                let mut split: Vec<String> = branch_line.split(" [").map(String::from).collect();
                split.pop().map(|l| format!("[{}", l))
            } else {
                None
            }
        })
        .flatten()
}

#[cfg(test)]
mod test {
    use super::*;

    fn to_string_vec(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|s| s.to_owned()).collect()
    }
    #[test]
    fn test_get_behindness_func() {
        let input1 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter [behind 1]"]);
        let input2 = to_string_vec(vec!["## master...origin/master [behind 2]"]);
        let input3 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter"]);
        let input4 = to_string_vec(vec!["## master...origin/master"]);

        let expected1 = String::from("[behind 1]");
        let expected2 = String::from("[behind 2]");

        assert_eq!(get_behindness(&input1), Some(expected1));
        assert_eq!(get_behindness(&input2), Some(expected2));
        assert_eq!(get_behindness(&input3), None);
        assert_eq!(get_behindness(&input4), None);
    }

    #[test]
    fn test_get_branch_func() {
        let input1 = to_string_vec(vec!["## master...origin/master"]);
        let input2 = to_string_vec(vec!["## mas## ter...origin/mas## ter"]);
        let input3 = to_string_vec(vec!["## mas...## ter...origin/mas...## ter"]);
        let input4 = to_string_vec(vec!["## mas...[ter...origin/mas...[ter [behind 1]"]);
        let input5 = to_string_vec(vec!["## master...origin/master [behind 1]"]);

        let expected1 = String::from("master");
        let expected2 = String::from("mas## ter");
        let expected3 = String::from("mas...## ter");
        let expected4 = String::from("mas...[ter");
        let expected5 = String::from("master");

        assert_eq!(get_branch(&input1), Some(expected1));
        assert_eq!(get_branch(&input2), Some(expected2));
        assert_eq!(get_branch(&input3), Some(expected3));
        assert_eq!(get_branch(&input4), Some(expected4));
        assert_eq!(get_branch(&input5), Some(expected5));
    }
}
