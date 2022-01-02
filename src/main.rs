use clap::{App, Arg};
use reqwest::header;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("ghchdest")
        .version("0.1")
        .author("Kenshi Kamata<kenshi.kamata@gmail.com>")
        .about("Does awesome things")
        .arg(
            Arg::new("token")
                .long("token")
                .value_name("GHP_TOKEN")
                .help("github personal access token to call GitHub API")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("repo")
                .long("repo")
                .value_name("ORG/REPO")
                .help("repository to change")
                .required(true)
                .value_delimiter('/'),
        )
        .arg(
            Arg::new("base")
                .long("base")
                .value_name("REF")
                .help("base branch or PR")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::new("targets")
                .long("targets")
                .help("target branchs to change")
                .value_delimiter(',')
                .required(true),
        )
        .get_matches();
    // get github private access token
    let token = matches.value_of("token").unwrap();
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .user_agent("ghchdest/0.1")
        .build()?;

    // get repoistory info org, repo
    let repo_name = matches.values_of("repo").unwrap().collect::<Vec<&str>>();
    if repo_name.len() != 2 {
        // print error and exit
        println!(
            "invalid arg. style must be OWNER/REPO {:#?}",
            repo_name.len()
        );
    }
    let owner = repo_name[0];
    let repo = repo_name[1];
    let base = matches.value_of("base").unwrap();
    let req = HashMap::from([("query", format!("query {{ repository(owner:\"{}\", name:\"{}\") {{ pullRequest (number:{}) {{ headRefName }} }} }}", owner, repo, base))]);
    let resp = client
        .post("https://api.github.com/graphql")
        .json(&req)
        .header(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        )
        .send()
        .await?
        .json::<HashMap<String, HashMap<String, HashMap<String, HashMap<String, String>>>>>()
        .await?;
    let base_branch = resp["data"]["repository"]["pullRequest"]["headRefName"].as_str();
    println!(
        "update target PR's base branch to {} #{} ",
        base_branch, base
    );

    let targets = matches.values_of("targets").unwrap().collect::<Vec<&str>>();
    println!("target PullRequests {:#?}", targets.join(", "));

    let req = HashMap::from([("base", base_branch)]);
    for pr in targets.iter() {
        // https://api.github.com/repos/octocat/hello-world/pulls/42
        let path = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr
        );
        let resp = client.patch(path).json(&req).send().await?;
        if resp.status().is_success() {
            println!("success to update {} base branch to {}", pr, base_branch)
        } else {
            println!("failed to update {} base branch to {}", pr, base_branch)
        }
    }
    Ok(())
}
