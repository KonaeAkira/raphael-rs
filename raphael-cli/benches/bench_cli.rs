use std::path::PathBuf;
use std::process::{Command, Stdio};

struct BenchCase {
    name: &'static str,
    args: &'static str,
}

const CASES: &[BenchCase] = &[
    BenchCase {
        name: "pactmaker_3240_3130_600", // Minimum required stats.
        args: "solve --recipe-id 34961 --stats 3240 3130 600 --level 90 --manipulation",
    },
    BenchCase {
        name: "pactmaker_3240_3130_600_no_manipulation", // Minimum required stats.
        args: "solve --recipe-id 34961 --stats 3240 3130 600 --level 90",
    },
    BenchCase {
        name: "rare_tacos_4900_4800_620", // Raphael default stats.
        args: "solve --recipe-id 35829 --stats 4900 4800 620 --level 100 --manipulation",
    },
    BenchCase {
        name: "rare_tacos_4900_4800_620_backload_progress", // Raphael default stats.
        args: "solve --recipe-id 35829 --stats 4900 4800 620 --level 100 --manipulation --backload-progress",
    },
    BenchCase {
        name: "rare_tacos_4900_4800_620_adversarial", // Raphael default stats.
        args: "solve --recipe-id 35829 --stats 4900 4800 620 --level 100 --manipulation --adversarial",
    },
    BenchCase {
        name: "courtly_lover_5811_5461_649", // Teamcraft 7.4 High Tier Meld.
        args: "solve --recipe-id 37839 --stats 5811 5461 649 --level 100 --manipulation",
    },
    BenchCase {
        name: "courtly_lover_5811_5461_649_specialist", // Teamcraft 7.4 High Tier Meld.
        args: "solve --recipe-id 37839 --stats 5811 5461 649 --level 100 --manipulation --heart-and-soul --quick-innovation",
    },
    BenchCase {
        name: "high_memory_default_settings_1", // https://github.com/KonaeAkira/raphael-rs/issues/292#issue-3776371475
        args: "solve --recipe-id 37839 --stats 5811 5576 776 --level 100 --manipulation --initial-quality 5221",
    },
    BenchCase {
        name: "high_memory_default_settings_2", // https://github.com/KonaeAkira/raphael-rs/issues/292#issuecomment-3852925964
        args: "solve --recipe-id 37520 --stats 5855 5424 776 --level 100 --manipulation",
    },
];

fn build_cli() -> PathBuf {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    Command::new("cargo")
        .args(["build", "--release", "--package", "raphael-cli"])
        .current_dir(&workspace_root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to build raphael-cli");
    workspace_root.join("target/release/raphael-cli")
}

fn main() {
    let cli = build_cli();

    println!(
        "{:<42}{:>10}{:>14}{:>14}",
        "Name", "Time", "Memory", "Quality"
    );
    println!("{}", "-".repeat(80));

    for case in CASES {
        let child = Command::new(&cli)
            .args(case.args.split_ascii_whitespace())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn raphael-cli");
        let child_pid = child.id();

        let monitor = std::thread::spawn(move || {
            let mut peak_rss: usize = 0;
            while let Ok(status) = std::fs::read_to_string(format!("/proc/{child_pid}/status")) {
                for line in status.lines() {
                    if let Some(rest) = line.strip_prefix("VmRSS:")
                        && let Ok(kb) = rest.trim().trim_end_matches(" kB").parse::<usize>()
                    {
                        peak_rss = peak_rss.max(kb * 1024);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            peak_rss
        });

        let start = std::time::Instant::now();
        let result = child
            .wait_with_output()
            .expect("Failed to wait for raphael-cli");
        let elapsed = start.elapsed();
        let peak_rss = monitor.join().unwrap();

        let stdout = String::from_utf8_lossy(&result.stdout);
        let quality = stdout
            .lines()
            .find_map(|line| line.strip_prefix("Quality: "))
            .unwrap_or_default();

        println!(
            "{:<42}{:>10}{:>14}{:>14}",
            case.name,
            format!("{:.2} s", elapsed.as_secs_f64()),
            format!("{:.2} MiB", peak_rss as f64 / (1 << 20) as f64),
            quality,
        );
    }
}
