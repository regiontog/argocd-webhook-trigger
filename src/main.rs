use git2::{DiffOptions, Repository};
fn main() {
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    // Should always be HEAD for any given local commit yeah?
    let git_ref = "HEAD";

    let commit = repo.head().unwrap().peel_to_commit().unwrap();

    // New commit sha
    // git show --no-patch --pretty=%H
    let after = commit.id().to_string();

    // Prev commit sha
    // git show --no-patch --pretty=%P
    let parent = commit.parent(0);
    let before = parent
        .as_ref()
        .map(|o| o.id().to_string())
        .unwrap_or(after.clone());

    // List of modified files
    // git diff --name-only HEAD^1
    let modified = repo
        .diff_tree_to_tree(
            parent.ok().and_then(|p| p.tree().ok()).as_ref(),
            commit.tree().ok().as_ref(),
            Some(&mut DiffOptions::new()),
        )
        .unwrap();

    modified
        .print(git2::DiffFormat::NameOnly, |delta, hunk, line| {
            println!("{:?}", delta.old_file().path());
            true
        })
        .unwrap();

    // No real equivalent for local repo, but maybe the active(HEAD) branch does the trick?
    let default_branch = repo.head().unwrap();

    // What ArgoCD knows the git repo that was modified as
    let repo = "";

    println!("{}", git_ref);
    println!("{}", before);
    println!("{}", after);
    println!("{}", "");
    println!("{}", default_branch.name().unwrap());
    println!("{}", repo);
}
