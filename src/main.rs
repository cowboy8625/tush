use clap::Parser;
use walkdir::WalkDir;
mod arguments;
mod parser;
mod posting;
use bat::PrettyPrinter;
use std::io::Write;

// 1. Show all Issues with Bat and offer a choice to post it

fn get_ignore_() -> Vec<String> {
    let mut list: Vec<String> = std::fs::read_to_string(".gitignore")
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .map(|s| {
            if s.starts_with("/") {
                format!(".{s}")
            } else {
                s
            }
        })
        .collect();
    list.push("./.git".to_string());
    list.push("./.gitignore".to_string());
    list.push("./Cargo.toml".to_string());
    list.push("./Cargo.lock".to_string());
    list
}

fn is_same_path(a: &str, b: &str) -> bool {
    let (Ok(lhs), Ok(rhs)) = (std::fs::canonicalize(a), std::fs::canonicalize(b)) else {
        return false;
    };
    rhs.as_path()
        .starts_with(lhs.as_path().to_string_lossy().to_string().as_str())
}

fn main() {
    let owner = "cowboy8625";
    let repo = "test-tush";
    let branch = "master";

    let ignore_list = get_ignore_();

    let args = arguments::Cli::parse();

    let dir_path = &args.path;

    let files = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.file_type()
                .is_file()
                .then(|| e.path().to_string_lossy().to_string())
        })
        .filter(|path| !ignore_list.iter().any(|s| is_same_path(s, path)))
        .collect::<Vec<_>>();

    for filename in files {
        let expect_error_message = format!("failed to read file: {}", filename);
        let src = std::fs::read_to_string(filename.clone()).expect(&expect_error_message);
        let tokens = parser::parser_file(&src);
        let repo_issues = posting::get_all_issues(owner, repo);
        for token in tokens {
            let issue = posting::Issue::from((token, owner, repo, branch, filename.as_str()));
            if repo_issues.contains(&issue) {
                continue;
            }
            let string_issue = issue.to_string();
            let bytes = string_issue.as_bytes();

            PrettyPrinter::new()
                .input_from_bytes(&bytes)
                .language("markdown")
                .line_numbers(true)
                .grid(true)
                .print()
                .expect("Failed to print markdown");

            print!("Do you want to post this issue? [y/N]: ");
            std::io::stdout().flush().expect("Failed to flush stdout");
            let mut command = String::new();
            std::io::stdin()
                .read_line(&mut command)
                .expect("Failed to read line");
            match command.to_lowercase().trim() {
                "y" | "yes" => posting::post_issue(owner, repo, &issue),
                _ => {}
            }
        }
    }
}
