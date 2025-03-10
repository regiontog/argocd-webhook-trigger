use git2::{DiffOptions, Repository};
use ureq::tls::TlsConfig;

fn main() {
    let mut insecure = false;

    let mut args = Vec::new();

    let mut arguments = std::env::args();

    let _ = arguments.next();

    for argument in arguments {
        if argument == "-k" {
            insecure = true;
            continue;
        }

        args.push(argument);
    }

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    // Should always be HEAD for any given local commit yeah?
    let git_ref = "HEAD";

    let commit = repo
        .head()
        .expect("repo to contain a commit")
        .peel_to_commit()
        .unwrap();

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
    let mut modified = Vec::new();

    repo.diff_tree_to_tree(
        parent.ok().and_then(|p| p.tree().ok()).as_ref(),
        commit.tree().ok().as_ref(),
        Some(&mut DiffOptions::new()),
    )
    .unwrap()
    .print(git2::DiffFormat::NameOnly, |delta, _, _| {
        if let Some(path) = delta.old_file().path() {
            modified.push(path.to_string_lossy().to_string());
        }

        true
    })
    .unwrap();

    // No real equivalent for local repo, but maybe the active(HEAD) branch does the trick?
    let default_branch = repo.head().unwrap();

    // What ArgoCD knows the git repo that was modified as
    let repo = &args[0];
    let argocd_server = &args[1];

    // Call ArgoCD pretending to be a GitHub Event
    // curl -H "Content-Type: application/json" -H 'X-GitHub-Event: push' --data '{ "ref": {{ .REF | toJson }}, "before": {{ .BEFORE | toJson }}, "after": {{ .AFTER | toJson }}, "repository": { "html_url": {{ .REPO | toJson }}, "default_branch": {{ .DEFAULT_BRANCH | toJson }} }, "commits": [ { "modified": [{{ .MODIFIED | toJson }}] } ] }' https://argocd.localho.st:8443/api/webhook
    let agent = ureq::Agent::config_builder()
        .tls_config(TlsConfig::builder().disable_verification(insecure).build())
        .build()
        .new_agent();

    println!(
        "{:?}",
        agent
            .post(&format!("{}/api/webhook", argocd_server))
            .header("X-GitHub-Event", "push")
            .send_json(&GitInfo {
                _ref: git_ref.to_string(),
                before,
                after,
                repository: Repo {
                    html_url: repo.to_string(),
                    default_branch: default_branch.name().unwrap().to_string(),
                },
                commits: vec![Commit {
                    modified: vec![modified.join("\n")],
                }],
            })
            .expect("to send webhook")
    );
}

#[derive(serde::Serialize)]
struct GitInfo {
    #[serde(rename = "ref")]
    _ref: String,
    before: String,
    after: String,
    repository: Repo,
    commits: Vec<Commit>,
}

#[derive(serde::Serialize)]
struct Repo {
    html_url: String,
    default_branch: String,
}

#[derive(serde::Serialize)]
struct Commit {
    modified: Vec<String>,
}
