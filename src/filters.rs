//! Program-aware output filters.
//!
//! Every command runs through a filter. High-volume commands (search, build,
//! test, VCS, listings, containers, packages) get dedicated compression;
//! everything else gets an aggressive generic pass. Filters preserve the
//! signal an agent needs (errors, locations, results, changed paths) and drop
//! noise (progress, decoration, repeated frames, context lines). Originals are
//! recoverable from the ledger, so filters are deliberately aggressive.

use crate::squeeze;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::BTreeMap;

lazy_static! {
    static ref ANSI: Regex = Regex::new(r"\x1b\[[0-9;?]*[ -/]*[@-~]").unwrap();
    static ref GREP_LOC: Regex = Regex::new(r"^([^:]+):(\d+):(.*)$").unwrap();
    static ref GREP_FILE: Regex = Regex::new(r"^([^:]+):(.*)$").unwrap();
    static ref ERR: Regex = Regex::new(
        r"(?i)\b(error|warning|failed|failure|panic(ked)?|exception|fatal|cannot|undefined reference|unresolved|not found|denied|traceback)\b"
    ).unwrap();
    static ref LOC: Regex = Regex::new(r"(^\s*-->\s)|(^\s*at\s)|(:\d+:\d+)|(\bline\s+\d+)").unwrap();
    static ref RESULT: Regex = Regex::new(
        r"(?i)(\d+\s+(passed|passing|failed|failing|ok|errors?|skipped|ignored))|(test result:)|(\bFAILED\b)|(\bPASSED\b)|(\d+\s+(tests?|examples?|assertions?))|(BUILD (SUCCESS|FAILURE))|(✓|✗|×)"
    ).unwrap();
    static ref NOISE: Regex = Regex::new(
        r"(?i)^\s*(compiling|downloading|downloaded|installing|fetching|updating|building|resolving|reading|writing|reused|locking|preparing|added \d|packages in|\$ )"
    ).unwrap();
}

fn strip(s: &str) -> String {
    ANSI.replace_all(s, "").into_owned()
}

pub fn apply(prog: &str, args: &[String], raw: &str) -> String {
    let base = prog.rsplit(['/', '\\']).next().unwrap_or(prog);
    let clean = strip(raw);
    let sub = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());

    // subcommand-aware routing for multiplexed CLIs (logs are the biggest win)
    match (base, sub) {
        ("docker" | "podman" | "kubectl" | "oc", Some("logs")) => return logs(&clean),
        ("kubectl" | "oc", Some("describe")) => return generic(&clean),
        _ => {}
    }

    match base {
        "git" => git(args, &clean),
        "grep" | "rg" | "ag" | "ack" | "ripgrep" => search(&clean),
        "cargo" => cargo(args, &clean),
        "go" => go(args, &clean),
        "npm" | "pnpm" | "yarn" | "bun" | "deno" => node_pm(args, &clean),
        "make" | "gradle" | "gradlew" | "mvn" | "ninja" | "cmake" | "bazel" | "meson"
        | "scons" | "msbuild" | "xcodebuild" => build(&clean),
        "gcc" | "g++" | "clang" | "clang++" | "cc" | "c++" | "rustc" | "javac"
        | "swiftc" | "tsc" => compiler(&clean),
        "eslint" | "biome" | "prettier" | "ruff" | "mypy" | "flake8" | "pylint"
        | "clippy" | "golangci-lint" | "shellcheck" | "stylelint" | "rubocop"
        | "black" | "isort" | "hadolint" => linter(&clean),
        "pytest" | "jest" | "vitest" | "mocha" | "rspec" | "phpunit" | "tox"
        | "nose" | "ava" | "karma" | "playwright" | "cypress" => test(&clean),
        "dotnet" => dotnet(args, &clean),
        "ls" | "find" | "fd" | "tree" | "exa" | "eza" | "lsd" => listing(&clean),
        "docker" | "podman" | "kubectl" | "oc" | "helm" | "nerdctl" | "crictl" => table(&clean),
        "terraform" | "tofu" | "pulumi" | "ansible" | "ansible-playbook"
        | "terragrunt" | "cdk" => infra(&clean),
        "systemctl" | "service" => systemd(&clean),
        "journalctl" | "dmesg" | "logread" => logs(&clean),
        "ps" | "df" | "du" | "free" | "top" | "htop" | "vmstat" | "iostat"
        | "lsblk" | "lscpu" | "mount" => table(&clean),
        "netstat" | "ss" | "lsof" | "ip" | "ifconfig" | "route" | "arp" => table(&clean),
        "ping" | "traceroute" | "tracepath" | "mtr" | "dig" | "nslookup" | "host" => netdiag(&clean),
        "strace" | "ltrace" | "dtruss" | "dtrace" => logs(&clean),
        "rsync" | "scp" | "sftp" | "rclone" => transfer(&clean),
        "tar" | "unzip" | "zip" | "7z" | "7za" | "gzip" | "gunzip" | "unrar" => archive(&clean),
        "nmap" | "masscan" => nmap(&clean),
        "valgrind" => compiler(&clean),
        "man" | "info" | "tldr" => passthrough_cap(&clean, 300),
        "ffmpeg" | "ffprobe" | "yt-dlp" | "youtube-dl" | "wget2" => tail_only(&clean, 4),
        "pip" | "pip3" | "poetry" | "gem" | "bundle" | "composer" | "apt" | "apt-get"
        | "brew" | "dnf" | "yum" | "pacman" | "snap" | "dpkg" | "rpm" | "conda"
        | "nix" | "cabal" | "opam" => pkg(&clean),
        "aws" | "gcloud" | "az" | "doctl" | "eksctl" | "ibmcloud" | "oci"
        | "linode-cli" | "flyctl" | "fly" | "heroku" | "vercel" | "wrangler" => cloud(&clean),
        "psql" | "mysql" | "mariadb" | "sqlite3" | "sqlite" | "mongosh" | "mongo"
        | "redis-cli" | "cqlsh" | "clickhouse-client" | "duckdb" | "cockroach" => dbclient(&clean),
        "cat" | "head" | "tail" | "bat" | "less" | "more" => passthrough_cap(&clean, 400),
        "curl" | "wget" | "http" | "httpie" => http(&clean),
        "diff" | "delta" | "colordiff" => diffcmd(&clean),
        "jq" | "yq" | "json" | "fx" => json_compact(&clean),
        "env" | "printenv" | "set" | "export" => env(&clean),
        _ => generic(&clean),
    }
}

// --- search: the headline. group matches by file, dedupe, cap. -------------
fn search(raw: &str) -> String {
    let mut by_file: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut plain: Vec<String> = Vec::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Some(c) = GREP_LOC.captures(line) {
            let file = c[1].to_string();
            let entry = format!("{}: {}", &c[2], c[3].trim());
            by_file.entry(file).or_default().push(cap_line(&entry, 200));
        } else if let Some(c) = GREP_FILE.captures(line) {
            let file = c[1].to_string();
            by_file.entry(file).or_default().push(cap_line(c[2].trim(), 200));
        } else {
            plain.push(cap_line(line, 200));
        }
    }
    if by_file.is_empty() {
        plain.dedup();
        return plain.into_iter().take(200).collect::<Vec<_>>().join("\n");
    }
    // Sort files by match count (most relevant first) and bound the whole
    // result: at most N files shown, a few matches each. Originals are in the
    // ledger, so aggressive caps lose nothing recoverable.
    let per_file_cap = 4usize;
    let file_cap = 30usize;
    let mut files: Vec<(String, Vec<String>)> = by_file.into_iter().collect();
    for (_, m) in files.iter_mut() {
        m.dedup();
    }
    files.sort_by(|a, b| b.1.len().cmp(&a.1.len()).then(a.0.cmp(&b.0)));

    let shown_files = files.len().min(file_cap);
    let hidden_files = files.len() - shown_files;
    let hidden_matches: usize = files.iter().skip(file_cap).map(|(_, m)| m.len()).sum();

    let mut out = Vec::new();
    for (file, matches) in files.iter().take(file_cap) {
        let total = matches.len();
        out.push(format!("{file} ({total})"));
        for m in matches.iter().take(per_file_cap) {
            out.push(format!("  {}", cap_line(m, 120)));
        }
        if total > per_file_cap {
            out.push(format!("  … +{} more", total - per_file_cap));
        }
    }
    if hidden_files > 0 {
        out.push(format!("… +{hidden_files} more files ({hidden_matches} matches)"));
    }
    let _ = shown_files;
    out.join("\n")
}

fn cap_line(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!("{}…", s.chars().take(n).collect::<String>())
    } else {
        s.to_string()
    }
}

// --- git -------------------------------------------------------------------
fn git(args: &[String], clean: &str) -> String {
    let sub = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    match sub {
        Some("status") => {
            let changes: Vec<String> = clean
                .lines()
                .filter_map(|l| {
                    let t = l.trim_start();
                    if t.starts_with("modified:") || t.starts_with("new file:")
                        || t.starts_with("deleted:") || t.starts_with("renamed:")
                        || t.starts_with("both modified:")
                    {
                        Some(t.to_string())
                    } else if l.len() >= 2 && l.as_bytes()[0] != b' ' && &l[..2] != "##"
                        && l.chars().nth(2) == Some(' ') && !l.starts_with("On ")
                    {
                        Some(l.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            if changes.is_empty() {
                "clean — nothing to commit".into()
            } else {
                changes.join("\n")
            }
        }
        Some("log") => {
            let mut out = Vec::new();
            let mut hash = String::new();
            for line in clean.lines() {
                if let Some(h) = line.strip_prefix("commit ") {
                    hash = h.chars().take(8).collect();
                } else if !hash.is_empty()
                    && !line.starts_with("Author:")
                    && !line.starts_with("Date:")
                    && !line.starts_with("Merge:")
                    && !line.trim().is_empty()
                {
                    out.push(format!("{hash} {}", line.trim()));
                    hash.clear();
                }
            }
            if out.is_empty() { generic(clean) } else { out.join("\n") }
        }
        Some("diff") | Some("show") => clean
            .lines()
            .filter(|l| {
                l.starts_with("diff ") || l.starts_with("@@")
                    || (l.starts_with('+') && !l.starts_with("+++"))
                    || (l.starts_with('-') && !l.starts_with("---"))
                    || l.starts_with("+++") || l.starts_with("---")
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Some("branch") | Some("tag") | Some("remote") => clean
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => generic(clean),
    }
}

// --- build / compile -------------------------------------------------------
fn keep_diag(clean: &str) -> Vec<&str> {
    let mut keep: Vec<&str> = clean
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !NOISE.is_match(l) && (ERR.is_match(l) || LOC.is_match(l) || RESULT.is_match(l))
        })
        .collect();
    keep.dedup();
    keep
}

fn build(clean: &str) -> String {
    let keep = keep_diag(clean);
    if keep.is_empty() {
        return tail_summary(clean, 2);
    }
    keep.join("\n")
}

fn cargo(args: &[String], clean: &str) -> String {
    let sub = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    if matches!(sub, Some("test") | Some("nextest")) {
        return test(clean);
    }
    build(clean)
}

fn go(args: &[String], clean: &str) -> String {
    let sub = args.iter().find(|a| !a.starts_with('-')).map(|s| s.as_str());
    if sub == Some("test") {
        return test(clean);
    }
    build(clean)
}

fn node_pm(args: &[String], clean: &str) -> String {
    let joined = args.join(" ");
    if joined.contains("test") {
        return test(clean);
    }
    if joined.contains("install") || joined.contains(" i ") || joined.contains("add") {
        return pkg(clean);
    }
    build(clean)
}

fn linter(clean: &str) -> String {
    let keep = keep_diag(clean);
    if keep.is_empty() {
        "clean — no issues".into()
    } else {
        keep.join("\n")
    }
}

// --- tests: failures + summary only ----------------------------------------
fn test(clean: &str) -> String {
    lazy_static! {
        // pytest/jest/etc. session headers and progress chrome — pure noise
        static ref HDR: Regex = Regex::new(
            r"(?i)^(platform |rootdir|plugins:|cachedir|collecting|configfile|test session starts|=+ test|PASS |RUNS |Test Suites:|Tests:|Snapshots:|Time:|Ran all)"
        ).unwrap();
        static ref DOTS: Regex = Regex::new(r"^[\s.sxFE✓✗×·]+$").unwrap();
    }
    let mut keep: Vec<&str> = clean
        .lines()
        .filter(|l| {
            let t = l.trim();
            if t.is_empty() || NOISE.is_match(l) || HDR.is_match(t) || DOTS.is_match(l) {
                return false;
            }
            RESULT.is_match(l)
                || ERR.is_match(l)
                || t.starts_with("FAIL")
                || t.starts_with("✗")
                || t.starts_with("×")
                || t.contains("assert")
                || LOC.is_match(l)
        })
        .collect();
    keep.dedup();
    if keep.is_empty() {
        return tail_summary(clean, 2);
    }
    keep.join("\n")
}

// --- listings: names only --------------------------------------------------
fn listing(clean: &str) -> String {
    let names: Vec<String> = clean
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with("total "))
        .map(|l| {
            // `ls -l`/`find`: keep just the name; plain `ls`: keep the token
            if l.len() > 40 && (l.starts_with('-') || l.starts_with('d') || l.starts_with('l')) {
                l.split_whitespace().skip(8).collect::<Vec<_>>().join(" ")
            } else {
                l.trim().to_string()
            }
        })
        .filter(|l| !l.is_empty() && l != "." && l != "..")
        .collect();
    // names are short — pack them space-separated to save the per-line overhead
    if names.iter().all(|n| !n.contains(' ')) {
        names.join("  ")
    } else {
        names.join("\n")
    }
}

// --- tabular output: keep header + rows, collapse column padding -----------
fn table(clean: &str) -> String {
    lazy_static! {
        static ref PAD: Regex = Regex::new(r" {2,}|\t+").unwrap();
    }
    clean
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.chars().all(|c| "+-=|_ ".contains(c))
        })
        .map(|l| PAD.replace_all(l.trim_end(), " ").into_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

// --- package managers: result lines only -----------------------------------
fn pkg(clean: &str) -> String {
    let keep: Vec<&str> = clean
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty()
                && !NOISE.is_match(l)
                && (ERR.is_match(l)
                    || RESULT.is_match(l)
                    || t.contains("installed")
                    || t.contains("added")
                    || t.contains("removed")
                    || t.contains("up to date")
                    || t.contains("Successfully"))
        })
        .collect();
    if keep.is_empty() {
        tail_summary(clean, 2)
    } else {
        keep.join("\n")
    }
}

fn passthrough_cap(clean: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = clean.lines().collect();
    if lines.len() <= max_lines {
        return generic(clean);
    }
    let head = lines.iter().take(max_lines * 3 / 4).cloned().collect::<Vec<_>>();
    let tail = lines.iter().rev().take(max_lines / 4).rev().cloned().collect::<Vec<_>>();
    format!(
        "{}\n  … [obelisk: {} of {} lines elided]\n{}",
        head.join("\n"),
        lines.len() - max_lines,
        lines.len(),
        tail.join("\n")
    )
}

fn tail_summary(clean: &str, n: usize) -> String {
    let lines: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
    let len = lines.len();
    lines.into_iter().skip(len.saturating_sub(n)).collect::<Vec<_>>().join("\n")
}

// --- compilers: errors + locations, drop everything else ------------------
fn compiler(clean: &str) -> String {
    let keep = keep_diag(clean);
    if keep.is_empty() {
        "compiled — no diagnostics".into()
    } else {
        keep.join("\n")
    }
}

fn dotnet(args: &[String], clean: &str) -> String {
    let joined = args.join(" ");
    if joined.contains("test") {
        test(clean)
    } else {
        build(clean)
    }
}

// --- logs: collapse repetitive lines (timestamps/ids vary) with counts. ----
// The single biggest win on container/journal logs, which are mostly the same
// message repeated thousands of times.
fn logs(clean: &str) -> String {
    lazy_static! {
        static ref NUM: Regex = Regex::new(r"\d").unwrap();
        // strip a leading ISO/syslog timestamp so otherwise-identical lines group
        static ref TS: Regex = Regex::new(
            r"^\s*(\[?\d{4}-\d{2}-\d{2}[T ][\d:.,]+Z?\]?|\w{3}\s+\d+\s[\d:]+|\[\d[\d:.]*\])\s*"
        ).unwrap();
    }
    let norm = |l: &str| -> String {
        let l = TS.replace(l, "");
        NUM.replace_all(&l, "#").into_owned()
    };
    let lines: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let key = norm(lines[i]);
        let mut j = i + 1;
        while j < lines.len() && norm(lines[j]) == key {
            j += 1;
        }
        let count = j - i;
        if count > 1 {
            out.push(format!("{}  [obelisk: ×{count}]", lines[i]));
        } else {
            out.push(lines[i].to_string());
        }
        i = j;
    }
    // also surface any error lines that were buried in non-consecutive runs
    if out.len() > 400 {
        let errs: Vec<String> = out.iter().filter(|l| ERR.is_match(l)).cloned().collect();
        let head: Vec<String> = out.into_iter().take(300).collect();
        let mut res = head;
        if !errs.is_empty() {
            res.push(format!("… [obelisk: {} error lines below]", errs.len()));
            res.extend(errs.into_iter().take(100));
        }
        return res.join("\n");
    }
    out.join("\n")
}

// --- infra plans: keep resource changes + summary, drop unchanged refresh. --
fn infra(clean: &str) -> String {
    lazy_static! {
        static ref CHANGE: Regex = Regex::new(
            r"^\s*([+\-~!]|<=|->|#|Plan:|Apply complete|Destroy complete|Error:|Changes to|will be (created|destroyed|updated|replaced)|must be replaced|forces replacement|No changes)"
        ).unwrap();
    }
    let keep: Vec<&str> = clean
        .lines()
        .filter(|l| CHANGE.is_match(l) && !l.trim().is_empty())
        .collect();
    if keep.is_empty() {
        tail_summary(clean, 3)
    } else {
        keep.join("\n")
    }
}

// --- systemd: status essentials only ---------------------------------------
fn systemd(clean: &str) -> String {
    let keep: Vec<&str> = clean
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("Loaded:")
                || t.starts_with("Active:")
                || t.starts_with("Main PID:")
                || t.starts_with("Tasks:")
                || t.starts_with("Memory:")
                || t.starts_with("●")
                || ERR.is_match(l)
        })
        .collect();
    if keep.is_empty() {
        table(clean)
    } else {
        keep.join("\n")
    }
}

// --- network diagnostics: summary lines only -------------------------------
fn netdiag(clean: &str) -> String {
    lazy_static! {
        static ref KEEP: Regex = Regex::new(
            r"(?i)(packets transmitted|packet loss|round-trip|rtt min|min/avg/max|ANSWER SECTION|^Address:|^Name:|^;;|connect:|unreachable|timed out|statistics ---|hops max)"
        ).unwrap();
        // per-packet / per-hop chatter to drop on ping & friends
        static ref DROP: Regex =
            Regex::new(r"(?i)(icmp_seq|bytes from|Request timeout for icmp_seq|^\s*\d+\s+[\d.]+ ms)").unwrap();
    }
    let keep: Vec<&str> = clean
        .lines()
        .filter(|l| !l.trim().is_empty() && KEEP.is_match(l) && !DROP.is_match(l))
        .collect();
    if keep.is_empty() {
        tail_summary(clean, 4)
    } else {
        keep.join("\n")
    }
}

// --- plain `diff`: changed lines + hunk headers ----------------------------
fn diffcmd(clean: &str) -> String {
    clean
        .lines()
        .filter(|l| {
            l.starts_with("@@")
                || l.starts_with("+++")
                || l.starts_with("---")
                || l.starts_with("> ")
                || l.starts_with("< ")
                || (l.starts_with('+') && !l.starts_with("+++"))
                || (l.starts_with('-') && !l.starts_with("---"))
                || (!l.is_empty() && l.as_bytes()[0].is_ascii_digit()) // `Nc M` etc
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// --- env: one per line, values truncated -----------------------------------
fn env(clean: &str) -> String {
    clean
        .lines()
        .filter(|l| !l.trim().is_empty() && l.contains('='))
        .map(|l| match l.split_once('=') {
            Some((k, v)) if v.len() > 60 => format!("{k}={}…", &v[..60]),
            _ => l.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// --- JSON: collapse repeated array elements, truncate long scalars ---------
// The big win for cloud CLIs (`aws … describe-*`, `gcloud … --format=json`),
// which return huge arrays of near-identical objects.
fn json_compact(clean: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(clean.trim()) {
        Ok(v) => {
            let c = compact_json(&v);
            serde_json::to_string(&c).unwrap_or_else(|_| generic(clean))
        }
        Err(_) => generic(clean),
    }
}

fn compact_json(v: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Array(a) => {
            let keep = 2usize;
            if a.len() > keep + 1 {
                let mut out: Vec<Value> = a.iter().take(keep).map(compact_json).collect();
                out.push(Value::String(format!("… +{} more items", a.len() - keep)));
                Value::Array(out)
            } else {
                Value::Array(a.iter().map(compact_json).collect())
            }
        }
        Value::Object(o) => {
            Value::Object(o.iter().map(|(k, val)| (k.clone(), compact_json(val))).collect())
        }
        Value::String(s) if s.len() > 120 => {
            Value::String(format!("{}…", s.chars().take(120).collect::<String>()))
        }
        other => other.clone(),
    }
}

// --- cloud CLIs: JSON when present, else compact tables --------------------
fn cloud(clean: &str) -> String {
    let t = clean.trim_start();
    if t.starts_with('{') || t.starts_with('[') {
        json_compact(clean)
    } else {
        table(clean)
    }
}

// --- database clients: compact rows, cap, drop borders ---------------------
fn dbclient(clean: &str) -> String {
    let t = table(clean);
    let lines: Vec<&str> = t.lines().collect();
    let cap = 200usize;
    if lines.len() > cap {
        let mut out: Vec<String> = lines.iter().take(cap).map(|s| s.to_string()).collect();
        out.push(format!("… +{} more rows", lines.len() - cap));
        out.join("\n")
    } else {
        t
    }
}

// --- http clients: headers + a capped body --------------------------------
fn http(clean: &str) -> String {
    let t = clean.trim_start();
    if t.starts_with('{') || t.starts_with('[') {
        return json_compact(clean);
    }
    passthrough_cap(clean, 200)
}

// --- file transfers: drop per-file progress, keep the summary -------------
fn transfer(clean: &str) -> String {
    lazy_static! {
        static ref KEEP: Regex = Regex::new(
            r"(?i)(sent .* received|total size|speedup|bytes/sec|transferred|^Number of files|^Total transferred|error|failed|denied)"
        ).unwrap();
    }
    let keep: Vec<&str> = clean
        .lines()
        .filter(|l| !l.trim().is_empty() && KEEP.is_match(l))
        .collect();
    if keep.is_empty() {
        // fall back to a capped file list
        passthrough_cap(clean, 60)
    } else {
        keep.join("\n")
    }
}

// --- archives: a capped file listing --------------------------------------
fn archive(clean: &str) -> String {
    let lines: Vec<&str> = clean.lines().filter(|l| !l.trim().is_empty()).collect();
    let cap = 80usize;
    if lines.len() > cap {
        let mut out: Vec<String> = lines.iter().take(cap).map(|s| s.to_string()).collect();
        out.push(format!("… +{} more entries", lines.len() - cap));
        out.join("\n")
    } else {
        lines.join("\n")
    }
}

// --- nmap: open ports + host status only ----------------------------------
fn nmap(clean: &str) -> String {
    lazy_static! {
        static ref KEEP: Regex = Regex::new(
            r"(?i)(open|filtered\b|Nmap scan report|Host is|PORT\s+STATE|MAC Address|^\d+/(tcp|udp)|scanned in)"
        ).unwrap();
    }
    let keep: Vec<&str> = clean.lines().filter(|l| KEEP.is_match(l) && !l.trim().is_empty()).collect();
    if keep.is_empty() { tail_summary(clean, 3) } else { keep.join("\n") }
}

fn tail_only(clean: &str, n: usize) -> String {
    tail_summary(clean, n)
}

fn generic(clean: &str) -> String {
    squeeze::squeeze(clean, false).map(|s| s.text).unwrap_or_else(|_| clean.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(prog: &str, args: &[&str], raw: &str) -> String {
        let av: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        apply(prog, &av, raw)
    }

    #[test]
    fn search_groups_and_shrinks() {
        let mut raw = String::new();
        for i in 0..50 {
            raw.push_str(&format!("src/a.rs:{i}:    fn thing{i}() {{}}\n"));
        }
        for i in 0..50 {
            raw.push_str(&format!("src/b.rs:{i}:    fn other{i}() {{}}\n"));
        }
        let out = a("grep", &["-rn", "fn"], &raw);
        assert!(out.len() < raw.len() / 3, "should shrink a lot");
        assert!(out.contains("src/a.rs (50)"));
        assert!(out.contains("+46 more")); // per-file cap of 4
    }

    #[test]
    fn git_status_keeps_changes_drops_chrome() {
        let raw = "On branch main\nYour branch is up to date.\n\nChanges not staged for commit:\n  (use \"git add\")\n\tmodified:   src/main.rs\n\tmodified:   src/lib.rs\n";
        let out = a("git", &["status"], raw);
        assert!(out.contains("modified:   src/main.rs"));
        assert!(!out.contains("up to date"));
    }

    #[test]
    fn git_clean_is_one_line() {
        let raw = "On branch main\nnothing to commit, working tree clean\n";
        assert_eq!(a("git", &["status"], raw), "clean — nothing to commit");
    }

    #[test]
    fn logs_dedup_collapses_repeats() {
        let mut raw = String::new();
        for _ in 0..1000 {
            raw.push_str("Jun 30 12:00:00 host app[1]: INFO request handled status=200\n");
        }
        let out = logs(&raw);
        assert!(out.contains("×1000"));
        assert_eq!(out.lines().count(), 1);
    }

    #[test]
    fn logs_keeps_errors_among_noise() {
        let mut raw = String::new();
        for i in 0..300 {
            raw.push_str(&format!("ts host app: INFO ok {i}\n"));
        }
        raw.push_str("ts host app: ERROR disk full\n");
        let out = logs(&raw);
        assert!(out.to_lowercase().contains("error disk full"));
    }

    #[test]
    fn json_collapses_large_arrays() {
        let raw = r#"{"items":[{"id":1},{"id":2},{"id":3},{"id":4},{"id":5},{"id":6}]}"#;
        let out = a("aws", &["ec2", "describe"], raw);
        assert!(out.contains("more items"));
        assert!(out.len() < raw.len());
    }

    #[test]
    fn json_invalid_falls_back() {
        let out = json_compact("not json at all");
        assert!(out.contains("not json"));
    }

    #[test]
    fn build_keeps_errors_drops_progress() {
        let raw = "   Compiling foo v0.1.0\n   Compiling bar v0.2.0\nerror[E0433]: failed to resolve\n  --> src/x.rs:3:5\n   Compiling baz v0.3\n";
        let out = a("cargo", &["build"], raw);
        assert!(out.contains("error[E0433]"));
        assert!(!out.contains("Compiling foo"));
    }

    #[test]
    fn table_collapses_padding() {
        let raw = "NAME       STATUS     AGE\npod-a      Running    2d\npod-b      Pending    1h\n";
        let out = a("kubectl", &["get", "pods"], raw);
        assert!(out.contains("NAME STATUS AGE"));
        assert!(out.len() < raw.len());
    }

    #[test]
    fn unknown_command_still_compressed() {
        let raw = "\x1b[31mhello\x1b[0m\n\n\n\nworld\n";
        let out = a("some-random-tool", &[], raw);
        assert!(!out.contains('\x1b'));
    }

    #[test]
    fn diff_keeps_changes() {
        let raw = "diff --git a/x b/x\n@@ -1,3 +1,3 @@\n unchanged\n-old line\n+new line\n unchanged2\n";
        let out = a("git", &["diff"], raw);
        assert!(out.contains("+new line"));
        assert!(out.contains("-old line"));
        assert!(!out.contains("unchanged2"));
    }
}
