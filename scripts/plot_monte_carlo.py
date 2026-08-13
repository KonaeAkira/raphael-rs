#!/usr/bin/env python3
"""Plot static/dynamic Monte Carlo results from the test's full JSON-like log."""

from __future__ import annotations

import argparse
import csv
import re
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np

ROW_RE = re.compile(
    r"iteration=(?P<iteration>\d+) seed=(?P<seed>0x[0-9a-fA-F]+) "
    r"(?P<strategy>static|dynamic)=\{quality:(?P<quality>\d+), "
    r"success:(?P<success>true|false), steps:(?P<steps>\d+), "
    r"duration:(?P<duration>\d+), run_ms:(?P<run_ms>\d+)\}"
)


def parse_log(path: Path) -> list[dict[str, int | str | bool]]:
    rows: list[dict[str, int | str | bool]] = []
    for match in ROW_RE.finditer(path.read_text(encoding="utf-8", errors="replace")):
        row = match.groupdict()
        rows.append(
            {
                "iteration": int(row["iteration"]),
                "seed": row["seed"],
                "strategy": row["strategy"],
                "quality": int(row["quality"]),
                "success": row["success"] == "true",
                "steps": int(row["steps"]),
                "duration": int(row["duration"]),
                "run_ms": int(row["run_ms"]),
            }
        )
    counts = {name: sum(row["strategy"] == name for row in rows) for name in ("static", "dynamic")}
    if not rows or counts["static"] != counts["dynamic"]:
        raise SystemExit(f"incomplete log: parsed {counts}")
    return rows


def values(rows: list[dict], strategy: str, field: str) -> np.ndarray:
    return np.array([row[field] for row in rows if row["strategy"] == strategy])


def common_bins(lhs: np.ndarray, rhs: np.ndarray, *, integer: bool = False, count: int = 12):
    low = min(lhs.min(), rhs.min())
    high = max(lhs.max(), rhs.max())
    if integer:
        return np.arange(np.floor(low) - 0.5, np.ceil(high) + 1.5, 1)
    if low == high:
        return np.array([low - 0.5, high + 0.5])
    return np.linspace(low, high, count + 1)


def histogram(path: Path, static: np.ndarray, dynamic: np.ndarray, title: str, xlabel: str, bins) -> None:
    fig, ax = plt.subplots(figsize=(9, 5.5), layout="constrained")
    ax.hist(static, bins=bins, alpha=0.62, label=f"Static (mean {static.mean():.1f})", color="#4C78A8")
    ax.hist(dynamic, bins=bins, alpha=0.62, label=f"Dynamic (mean {dynamic.mean():.1f})", color="#F58518")
    ax.set(title=title, xlabel=xlabel, ylabel="Crafts")
    ax.grid(axis="y", alpha=0.25)
    ax.legend()
    fig.savefig(path, dpi=180)
    plt.close(fig)


def quality_candles(path: Path, static: np.ndarray, dynamic: np.ndarray, maximum: int) -> None:
    """Quartile candles with full-range whiskers and individual observations."""
    colors = ["#4C78A8", "#F58518"]
    datasets = [static, dynamic]
    labels = ["Static", "Dynamic"]
    rng = np.random.default_rng(6178)

    fig, ax = plt.subplots(figsize=(8, 6), layout="constrained")
    box = ax.boxplot(
        datasets,
        tick_labels=labels,
        widths=0.48,
        whis=(0, 100),
        showmeans=True,
        patch_artist=True,
        medianprops={"color": "white", "linewidth": 2.2},
        meanprops={
            "marker": "D",
            "markerfacecolor": "#222222",
            "markeredgecolor": "white",
            "markersize": 7,
        },
    )
    for patch, color in zip(box["boxes"], colors, strict=True):
        patch.set_facecolor(color)
        patch.set_alpha(0.82)
    for index, (data, color) in enumerate(zip(datasets, colors, strict=True), start=1):
        jitter = rng.uniform(-0.12, 0.12, len(data))
        ax.scatter(index + jitter, data, s=13, alpha=0.25, color=color, edgecolors="none")
        q1, median, q3 = np.percentile(data, [25, 50, 75])
        max_rate = np.mean(data >= maximum) * 100
        ax.text(
            index,
            data.min() - 180,
            f"min {data.min():.0f}\nQ1 {q1:.0f}\nmedian {median:.0f}\nQ3 {q3:.0f}\nmean {data.mean():.0f}\nmax-quality {max_rate:.0f}%",
            ha="center",
            va="top",
            fontsize=9,
        )

    ax.axhline(maximum, color="#B22222", linestyle="--", linewidth=1.5, label=f"Maximum ({maximum:,})")
    ax.set_title("Final quality: quartiles, range, and observations")
    ax.set_ylabel("Final quality")
    ax.set_ylim(min(static.min(), dynamic.min()) - 1400, maximum + 350)
    ax.grid(axis="y", alpha=0.25)
    ax.legend(loc="lower right")
    fig.savefig(path, dpi=180)
    plt.close(fig)


def paired_quality(path: Path, static: np.ndarray, dynamic: np.ndarray, maximum: int) -> None:
    """Show the paired result for each seed instead of losing pairing in a histogram."""
    fig, ax = plt.subplots(figsize=(8, 6), layout="constrained")
    for lhs, rhs in zip(static, dynamic, strict=True):
        color = "#2E8B57" if rhs > lhs else "#B22222" if rhs < lhs else "#777777"
        ax.plot([0, 1], [lhs, rhs], color=color, alpha=0.22, linewidth=1)
    ax.scatter(np.zeros(len(static)), static, color="#4C78A8", s=18, alpha=0.65, zorder=3)
    ax.scatter(np.ones(len(dynamic)), dynamic, color="#F58518", s=18, alpha=0.65, zorder=3)
    ax.axhline(maximum, color="#B22222", linestyle="--", linewidth=1.5, label=f"Maximum ({maximum:,})")
    ax.set_xticks([0, 1], ["Static", "Dynamic"])
    ax.set_xlim(-0.3, 1.3)
    ax.set_ylabel("Final quality")
    ax.set_title("Paired quality by condition seed")
    ax.grid(axis="y", alpha=0.25)
    ax.legend()
    fig.savefig(path, dpi=180)
    plt.close(fig)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("log", type=Path)
    parser.add_argument("--output-dir", type=Path, default=Path("monte-carlo-plots"))
    args = parser.parse_args()
    args.output_dir.mkdir(parents=True, exist_ok=True)

    rows = parse_log(args.log)
    with (args.output_dir / "runs.csv").open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)

    static_quality = values(rows, "static", "quality")
    dynamic_quality = values(rows, "dynamic", "quality")
    histogram(
        args.output_dir / "quality_distribution.png",
        static_quality,
        dynamic_quality,
        "Final quality distribution",
        "Final quality",
        common_bins(static_quality, dynamic_quality),
    )
    quality_candles(
        args.output_dir / "quality_candlestick.png",
        static_quality,
        dynamic_quality,
        maximum=12_000,
    )
    paired_quality(
        args.output_dir / "quality_paired.png",
        static_quality,
        dynamic_quality,
        maximum=12_000,
    )

    static_wall = values(rows, "static", "run_ms") / 1000
    dynamic_wall = values(rows, "dynamic", "run_ms") / 1000
    histogram(
        args.output_dir / "wall_time_distribution.png",
        static_wall,
        dynamic_wall,
        "Solver wall-time distribution",
        "Wall time per craft (seconds)",
        common_bins(static_wall, dynamic_wall),
    )

    static_duration = values(rows, "static", "duration")
    dynamic_duration = values(rows, "dynamic", "duration")
    histogram(
        args.output_dir / "craft_time_distribution.png",
        static_duration,
        dynamic_duration,
        "In-game craft-time distribution",
        "Action duration (seconds)",
        common_bins(static_duration, dynamic_duration, integer=True),
    )

    static_steps = values(rows, "static", "steps")
    dynamic_steps = values(rows, "dynamic", "steps")
    histogram(
        args.output_dir / "step_count_distribution.png",
        static_steps,
        dynamic_steps,
        "Craft step-count distribution",
        "Number of actions",
        common_bins(static_steps, dynamic_steps, integer=True),
    )

    for strategy in ("static", "dynamic"):
        strategy_rows = [row for row in rows if row["strategy"] == strategy]
        print(
            f"{strategy}: n={len(strategy_rows)}, "
            f"quality_mean={np.mean([row['quality'] for row in strategy_rows]):.1f}, "
            f"wall_mean_s={np.mean([row['run_ms'] for row in strategy_rows]) / 1000:.3f}, "
            f"craft_mean_s={np.mean([row['duration'] for row in strategy_rows]):.2f}"
        )


if __name__ == "__main__":
    main()
