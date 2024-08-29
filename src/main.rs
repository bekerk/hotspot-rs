use git2::{Repository, Time};
use std::{
    cmp,
    collections::HashMap,
    path::PathBuf,
    time::{self, SystemTime},
};

struct FixCommit {
    message: String,
    date: Time,
    files: Vec<PathBuf>,
}

fn main() {
    let mut hotspot_cache = HashMap::new();
    let mut fixes: Vec<FixCommit> = Vec::new();
    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    let mut revwalk = repo.revwalk().unwrap();
    let _ = revwalk.push_head();

    for commit in revwalk {
        let commit_id = commit.unwrap();
        let commit = repo.find_commit(commit_id).unwrap();
        let commit_message_lowercase = commit.message().unwrap().to_ascii_lowercase();
        if commit_message_lowercase.contains("bug") {
            let parent_tree = commit.parent(0).unwrap().tree().unwrap();
            let commit_tree = commit.tree().unwrap();
            let diff = repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&commit_tree), None)
                .unwrap();
            let fix = FixCommit {
                message: commit.message().unwrap().to_string(),
                date: commit.time(),
                files: diff
                    .deltas()
                    .map(|x| x.old_file().path().unwrap().to_path_buf())
                    .collect::<Vec<_>>(),
            };
            fixes.push(fix);
        }
    }
    let system_time_now: f64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_millis() as f64,
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    let mut oldest_commit_date: f64 = system_time_now as f64;

    for fix in &mut fixes {
        let a = fix.date.seconds() as f64 * 1000.0;
        if a < oldest_commit_date {
            oldest_commit_date = a;
        }
    }

    // println!("size: {}", fixes.len());

    for fix in &fixes {
        let current_fix_date = fix.date.seconds() as f64 * 1000.0 ;
        let _ = &fix.files.iter().for_each(|x| {
            let score = 1.0
                - ((system_time_now - current_fix_date) / (system_time_now - oldest_commit_date));
            *hotspot_cache.entry(x.to_owned()).or_insert(0.0) +=
                1.0 / (1.0 + ((-24.0 * score) + 24.0).exp());
        });
        // for file in fix {
        //     let score = 1; /* temporary */
        //
        // }
    }

    let mut hotspot_cache_sorted_vec: Vec<(&std::path::PathBuf, &f64)> =
        hotspot_cache.iter().collect();
    hotspot_cache_sorted_vec.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    println!("{:#?}", hotspot_cache_sorted_vec);
}
