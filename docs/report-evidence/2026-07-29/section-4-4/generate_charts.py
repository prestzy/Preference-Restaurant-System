"""Regenerate the two Section 4.4 charts from the recorded CSV results."""

from __future__ import annotations

import csv
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt


BASE_DIR = Path(__file__).resolve().parent
TEXT = "#303446"
GRID = "#ccd0da"
RED = "#e64553"
BLUE = "#1e66f5"
ORANGE = "#fe640b"


def read_rows(filename: str) -> list[dict[str, str]]:
    """Load one evidence CSV while preserving its report-facing labels."""

    with (BASE_DIR / filename).open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source))


def apply_report_style(axis: plt.Axes) -> None:
    """Apply a restrained, high-contrast style suitable for an FYP report."""

    axis.set_facecolor("white")
    axis.grid(axis="y", color=GRID, linewidth=0.8, alpha=0.7)
    axis.set_axisbelow(True)
    axis.spines["top"].set_visible(False)
    axis.spines["right"].set_visible(False)
    axis.tick_params(colors=TEXT)
    axis.xaxis.label.set_color(TEXT)
    axis.yaxis.label.set_color(TEXT)
    axis.title.set_color(TEXT)


def generate_coorder_chart() -> None:
    """Plot candidate ranks at zero, three, and five temporary co-orders."""

    rows = read_rows("coorder-impact-results.csv")
    figure, axis = plt.subplots(figsize=(10, 6), dpi=180)
    apply_report_style(axis)

    series = [
        ("Pair A", "Nasi Lemak (D01) -> Sambal Sotong (D07)", RED, "o"),
        ("Pair B", "Nasi Lemak (D01) -> Chicken Satay (D09)", BLUE, "s"),
    ]
    for pair, label, color, marker in series:
        selected = [row for row in rows if row["pair"] == pair]
        x_values = [int(row["added_coorders"]) for row in selected]
        ranks = [int(row["candidate_rank"]) for row in selected]
        axis.plot(
            x_values,
            ranks,
            label=label,
            color=color,
            marker=marker,
            markersize=8,
            linewidth=2.5,
        )
        for x_value, rank in zip(x_values, ranks):
            axis.annotate(
                f"Rank {rank}",
                (x_value, rank),
                xytext=(0, 10),
                textcoords="offset points",
                ha="center",
                color=color,
                fontsize=9,
                fontweight="bold",
            )

    axis.set_title(
        "Effect of Simulated Co-Orders on Candidate Rank",
        fontsize=15,
        fontweight="bold",
        pad=16,
    )
    axis.set_xlabel("Added temporary co-orders", fontsize=11)
    axis.set_ylabel("Candidate rank", fontsize=11)
    axis.set_xticks([0, 3, 5])
    axis.set_yticks(range(1, 8))
    # Rank 1 is the strongest result, so it is placed at the top of the graph.
    axis.set_ylim(7.6, 0.4)
    axis.legend(loc="lower right", frameon=True, facecolor="white")
    figure.text(
        0.5,
        0.015,
        "Note: A lower rank value represents a stronger recommendation position.",
        ha="center",
        color=TEXT,
        fontsize=9,
    )
    figure.tight_layout(rect=(0, 0.05, 1, 1))
    figure.savefig(
        BASE_DIR / "figure-4-9-coorder-rank.png",
        bbox_inches="tight",
        facecolor="white",
    )
    plt.close(figure)


def generate_method_chart() -> None:
    """Plot the Hit@3 percentages for the three controlled methods."""

    rows = read_rows("method-comparison-summary.csv")
    labels = [row["method"] for row in rows]
    rates = [float(row["hit_at_3_rate_percent"]) for row in rows]
    colors = [ORANGE, BLUE, RED]

    figure, axis = plt.subplots(figsize=(9, 6), dpi=180)
    apply_report_style(axis)
    bars = axis.bar(labels, rates, color=colors, width=0.62)
    axis.set_title(
        "Hit@3 Rate by Recommendation Method",
        fontsize=15,
        fontweight="bold",
        pad=16,
    )
    axis.set_ylabel("Hit@3 rate (%)", fontsize=11)
    axis.set_ylim(0, 110)
    axis.set_yticks(range(0, 101, 20))
    axis.tick_params(axis="x", labelrotation=0)
    for bar, rate in zip(bars, rates):
        axis.text(
            bar.get_x() + bar.get_width() / 2,
            rate + 2,
            f"{rate:.0f}%",
            ha="center",
            va="bottom",
            color=TEXT,
            fontsize=11,
            fontweight="bold",
        )

    figure.tight_layout()
    figure.savefig(
        BASE_DIR / "figure-4-11-method-hit-at-3.png",
        bbox_inches="tight",
        facecolor="white",
    )
    plt.close(figure)


if __name__ == "__main__":
    generate_coorder_chart()
    generate_method_chart()
