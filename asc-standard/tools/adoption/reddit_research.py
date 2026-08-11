#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import hashlib
import html
import json
import re
import subprocess
import time
from collections import Counter
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from urllib.parse import urlencode, urlparse

import yaml


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_CONFIG = REPO_ROOT / "data/adoption/reddit-research.yaml"
DEFAULT_OUTPUT = REPO_ROOT / "docs/generated/reddit-research"
CSV_NAME = "pain_points.csv"
JSON_NAME = "pain_points.json"
SUMMARY_NAME = "SUMMARY.md"

POST_LINK_RE = re.compile(
    r'<a[^>]+data-testid="post-title-text"[^>]+href="(?P<href>/r/(?P<subreddit>[^/]+)/comments/(?P<post_id>[^/]+)/[^"]*/?)"[^>]*>(?P<title>.*?)</a>',
    re.S,
)
COMMENT_OPEN_RE = re.compile(r"<shreddit-comment(?P<attrs>[^>]*)>", re.S)
ATTR_RE = re.compile(r'([A-Za-z_:][\w:.-]*)="(.*?)"')
TOTAL_COMMENTS_RE = re.compile(r'<shreddit-comment-tree-stats[^>]*total-comments="(?P<count>\d+)"')


@dataclass(frozen=True)
class QuerySpec:
    query_id: str
    query: str
    subreddit: str | None
    limit: int
    sort: str
    time_range: str
    max_comments: int
    theme_hints: tuple[str, ...]


@dataclass(frozen=True)
class ThreadRef:
    query_id: str
    query: str
    subreddit: str
    post_id: str
    title: str
    thread_url: str
    theme_hints: tuple[str, ...]


def load_yaml(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = yaml.safe_load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a mapping at the top level")
    return data


def canonical_url(url: str) -> str:
    if url.startswith("http://") or url.startswith("https://"):
        return url
    return f"https://www.reddit.com{url}"


def normalize_whitespace(text: str) -> str:
    return re.sub(r"\s+", " ", html.unescape(text or "")).strip()


def html_to_text(fragment: str) -> str:
    text = re.sub(r"<br\s*/?>", "\n", fragment, flags=re.I)
    text = re.sub(r"</p\s*>", "\n", text, flags=re.I)
    text = re.sub(r"<p[^>]*>", "", text, flags=re.I)
    text = re.sub(r"<[^>]+>", "", text)
    return normalize_whitespace(text)


def split_sentences(text: str) -> list[str]:
    normalized = normalize_whitespace(text)
    if not normalized:
        return []
    parts = re.split(r"(?<=[.!?])\s+|(?<=:)\s+", normalized)
    return [part.strip() for part in parts if part.strip()] or [normalized]


def lowercase_matches(text: str, terms: list[str]) -> list[str]:
    haystack = text.lower()
    matches: set[str] = set()
    for term in terms:
        pattern = re.compile(rf"(?<![A-Za-z0-9]){re.escape(term.lower())}(?![A-Za-z0-9])")
        if pattern.search(haystack):
            matches.add(term)
    return sorted(matches)


def extract_quote(text: str, pain_terms: list[str], max_chars: int) -> str:
    sentences = split_sentences(text)
    matching = [sentence for sentence in sentences if lowercase_matches(sentence, pain_terms)]
    quote = " ".join(matching[:2]) if matching else normalize_whitespace(text)
    if len(quote) <= max_chars:
        return quote
    trimmed = quote[: max_chars - 3].rstrip()
    cut = trimmed.rfind(" ")
    if cut > 80:
        trimmed = trimmed[:cut]
    return trimmed.rstrip(" ,;:") + "..."


def parse_attrs(attrs: str) -> dict[str, str]:
    return {key: html.unescape(value) for key, value in ATTR_RE.findall(attrs)}


def isoformat_or_empty(value: str | None) -> str:
    if not value:
        return ""
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(UTC).isoformat()
    except ValueError:
        return value


def slug_title(thread_url: str) -> str:
    parts = [part for part in urlparse(thread_url).path.strip("/").split("/") if part]
    if len(parts) >= 5:
        return normalize_whitespace(parts[4].replace("_", " ").replace("-", " "))
    return normalize_whitespace(thread_url)


def curl_fetch(url: str, user_agent: str, timeout_seconds: int) -> str:
    result = subprocess.run(
        [
            "curl",
            "-fsSL",
            "--max-time",
            str(timeout_seconds),
            "-A",
            user_agent,
            url,
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def query_specs(config: dict[str, Any]) -> list[QuerySpec]:
    default_limit = int(config.get("default_limit", 6))
    default_sort = str(config.get("default_sort", "relevance"))
    default_time = str(config.get("default_time", "all"))
    default_max_comments = int(config.get("default_max_comments", 4))
    specs: list[QuerySpec] = []
    for item in config.get("queries", []):
        specs.append(
            QuerySpec(
                query_id=str(item["id"]),
                query=str(item["query"]),
                subreddit=str(item["subreddit"]) if item.get("subreddit") else None,
                limit=int(item.get("limit", default_limit)),
                sort=str(item.get("sort", default_sort)),
                time_range=str(item.get("time", default_time)),
                max_comments=int(item.get("max_comments", default_max_comments)),
                theme_hints=tuple(item.get("theme_hints", [])),
            )
        )
    return specs


def search_url(spec: QuerySpec) -> str:
    params = {
        "q": spec.query,
        "sort": spec.sort,
        "t": spec.time_range,
    }
    if spec.subreddit:
        params["restrict_sr"] = "1"
        return f"https://www.reddit.com/r/{spec.subreddit}/search/?{urlencode(params)}"
    return f"https://www.reddit.com/search/?{urlencode(params)}"


def parse_search_results(document: str, query_id: str, query: str, theme_hints: tuple[str, ...], limit: int) -> list[ThreadRef]:
    threads: list[ThreadRef] = []
    seen: set[str] = set()
    for match in POST_LINK_RE.finditer(document):
        href = canonical_url(match.group("href"))
        if href in seen:
            continue
        seen.add(href)
        title = html_to_text(match.group("title"))
        threads.append(
            ThreadRef(
                query_id=query_id,
                query=query,
                subreddit=match.group("subreddit"),
                post_id=match.group("post_id"),
                title=title,
                thread_url=href,
                theme_hints=theme_hints,
            )
        )
        if len(threads) >= limit:
            break
    return threads


def search_threads(spec: QuerySpec, config: dict[str, Any]) -> list[ThreadRef]:
    html_doc = curl_fetch(
        search_url(spec),
        user_agent=str(config["user_agent"]),
        timeout_seconds=int(config.get("http_timeout_seconds", 20)),
    )
    time.sleep(float(config.get("sleep_seconds", 0.4)))
    allowed = {item.lower() for item in config.get("allowed_subreddits", [])}
    threads = parse_search_results(html_doc, spec.query_id, spec.query, spec.theme_hints, spec.limit * 2)
    if allowed:
        threads = [thread for thread in threads if thread.subreddit.lower() in allowed]
    return threads[: spec.limit]


def parse_seed_thread(seed: dict[str, Any]) -> ThreadRef | None:
    url = canonical_url(str(seed["url"]))
    path_parts = [part for part in urlparse(url).path.strip("/").split("/") if part]
    if len(path_parts) < 4 or path_parts[0] != "r" or path_parts[2] != "comments":
        return None
    subreddit = path_parts[1]
    post_id = path_parts[3]
    title = normalize_whitespace(seed.get("title", "")) or slug_title(url)
    return ThreadRef(
        query_id=f"seed:{post_id}",
        query=title,
        subreddit=subreddit,
        post_id=post_id,
        title=title,
        thread_url=url.rstrip("/") + "/",
        theme_hints=tuple(seed.get("theme_hints", [])),
    )


def comments_url(thread: ThreadRef) -> str:
    params = {
        "seeker-session": "true",
        "render-mode": "partial",
        "referer": "",
    }
    return f"https://www.reddit.com/svc/shreddit/comments/r/{thread.subreddit}/{thread.post_id}?{urlencode(params)}"


def comment_body_for_thing(document: str, thing_id: str) -> str:
    pattern = re.compile(
        rf'<div id="{re.escape(thing_id)}-post-rtjson-content"[^>]*>(?P<body>.*?)</div>\s*</div>',
        re.S,
    )
    match = pattern.search(document)
    if not match:
        return ""
    return html_to_text(match.group("body"))


def fetch_comments(thread: ThreadRef, config: dict[str, Any]) -> tuple[list[dict[str, Any]], int]:
    document = curl_fetch(
        comments_url(thread),
        user_agent=str(config["user_agent"]),
        timeout_seconds=int(config.get("http_timeout_seconds", 20)),
    )
    time.sleep(float(config.get("sleep_seconds", 0.4)))
    total_comments_match = TOTAL_COMMENTS_RE.search(document)
    total_comments = int(total_comments_match.group("count")) if total_comments_match else 0

    comments: list[dict[str, Any]] = []
    for match in COMMENT_OPEN_RE.finditer(document):
        attrs = parse_attrs(match.group("attrs"))
        if not attrs.get("thingId") or not attrs.get("permalink"):
            continue
        body_text = comment_body_for_thing(document, attrs["thingId"])
        if not body_text or body_text in {"[deleted]", "[removed]"}:
            continue
        comments.append(
            {
                "thing_id": attrs["thingId"],
                "author": attrs.get("author", ""),
                "score": int(attrs.get("score", "0") or 0),
                "created": isoformat_or_empty(attrs.get("created")),
                "permalink": canonical_url(attrs["permalink"]),
                "body_text": body_text,
            }
        )
    return comments, total_comments


def theme_lookup(config: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {theme["id"]: theme for theme in config.get("themes", [])}


def assign_themes(
    combined_text: str,
    theme_hints: tuple[str, ...],
    themes: dict[str, dict[str, Any]],
) -> list[str]:
    lowered = combined_text.lower()
    scores: dict[str, int] = {}
    for theme_id, theme in themes.items():
        keywords = theme.get("keywords", [])
        score = sum(1 for keyword in keywords if keyword.lower() in lowered)
        if theme_id in theme_hints:
            score += 2
        if score:
            scores[theme_id] = score
    if not scores:
        return list(theme_hints[:1])
    return [theme_id for theme_id, _ in sorted(scores.items(), key=lambda item: (-item[1], item[0]))[:3]]


def row_id(source_url: str, quote_text: str) -> str:
    digest = hashlib.sha256(f"{source_url}\n{quote_text}".encode("utf-8")).hexdigest()
    return digest[:16]


def build_row(
    *,
    thread: ThreadRef,
    comment: dict[str, Any],
    total_comments: int,
    config: dict[str, Any],
    themes: dict[str, dict[str, Any]],
) -> dict[str, Any] | None:
    domain_text = " ".join([thread.title, comment["body_text"], thread.subreddit])
    combined_text = " ".join([thread.title, comment["body_text"], thread.query, thread.subreddit])
    domain_matches = lowercase_matches(domain_text, list(config.get("domain_terms", [])))
    developer_matches = lowercase_matches(domain_text, list(config.get("developer_terms", [])))
    pain_matches = lowercase_matches(comment["body_text"], list(config.get("pain_terms", [])))
    if not domain_matches or not developer_matches or not pain_matches:
        return None

    quote = extract_quote(comment["body_text"], list(config.get("pain_terms", [])), int(config.get("max_quote_chars", 320)))
    if not quote:
        return None

    theme_ids = assign_themes(combined_text, thread.theme_hints, themes)
    theme_labels = [themes[theme_id]["label"] for theme_id in theme_ids if theme_id in themes]
    ergonomics_surfaces = [
        themes[theme_id]["ergonomics_surface"]
        for theme_id in theme_ids
        if theme_id in themes and themes[theme_id].get("ergonomics_surface")
    ]
    verbatim_status = "yes" if quote.lower() in comment["body_text"].lower() else "normalized_quote"
    source_fetch_url = comments_url(thread)
    return {
        "row_id": row_id(comment["permalink"], quote),
        "validation_status": "validated" if verbatim_status == "yes" else "normalized_quote",
        "query_id": thread.query_id,
        "query": thread.query,
        "subreddit": f"r/{thread.subreddit}",
        "thread_id": thread.post_id,
        "thread_title": thread.title,
        "thread_url": thread.thread_url,
        "thread_num_comments": total_comments,
        "source_kind": "comment",
        "source_id": comment["thing_id"],
        "source_author": comment["author"],
        "source_score": comment["score"],
        "source_created_utc": comment["created"],
        "source_url": comment["permalink"],
        "source_fetch_url": source_fetch_url,
        "quote_text": quote,
        "theme_ids": ";".join(theme_ids),
        "theme_labels": ";".join(theme_labels),
        "ergonomics_surfaces": ";".join(ergonomics_surfaces),
        "domain_matches": ";".join(domain_matches),
        "developer_matches": ";".join(developer_matches),
        "pain_matches": ";".join(pain_matches),
        "has_direct_permalink": "yes",
        "has_fetch_url": "yes",
        "has_verbatim_quote": verbatim_status,
        "has_domain_match": "yes",
        "has_developer_match": "yes",
        "has_pain_match": "yes",
    }


def collect_rows(config: dict[str, Any]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    themes = theme_lookup(config)
    threads: list[ThreadRef] = []
    stats: dict[str, Any] = {
        "generated_at_utc": datetime.now(tz=UTC).isoformat(),
        "queries": [],
        "query_failures": [],
        "seed_threads": 0,
        "fetched_threads": 0,
        "fetch_failures": [],
    }

    for spec in query_specs(config):
        try:
            results = search_threads(spec, config)
        except subprocess.CalledProcessError as exc:
            stats["query_failures"].append(
                {
                    "query_id": spec.query_id,
                    "query": spec.query,
                    "subreddit": spec.subreddit,
                    "error": str(exc),
                }
            )
            continue
        stats["queries"].append(
            {
                "query_id": spec.query_id,
                "query": spec.query,
                "subreddit": spec.subreddit,
                "results": len(results),
            }
        )
        threads.extend(results)

    for seed in config.get("seed_threads", []):
        parsed = parse_seed_thread(seed)
        if parsed is None:
            continue
        threads.append(parsed)
        stats["seed_threads"] += 1

    deduped_threads: dict[str, ThreadRef] = {}
    for thread in threads:
        deduped_threads.setdefault(thread.thread_url, thread)

    rows: list[dict[str, Any]] = []
    seen_rows: set[str] = set()
    max_comments = int(config.get("default_max_comments", 4))
    min_comment_score = int(config.get("min_comment_score", 0))
    for thread in deduped_threads.values():
        try:
            comments, total_comments = fetch_comments(thread, config)
        except subprocess.CalledProcessError as exc:
            stats["fetch_failures"].append(
                {
                    "thread_url": thread.thread_url,
                    "error": str(exc),
                }
            )
            continue
        stats["fetched_threads"] += 1
        for comment in comments[:max_comments]:
            if int(comment["score"]) < min_comment_score:
                continue
            row = build_row(
                thread=thread,
                comment=comment,
                total_comments=total_comments,
                config=config,
                themes=themes,
            )
            if not row or row["row_id"] in seen_rows:
                continue
            seen_rows.add(row["row_id"])
            rows.append(row)

    rows.sort(key=lambda row: (row["theme_labels"], row["subreddit"], row["thread_title"], row["source_url"]))
    stats["searched_threads"] = sum(item["results"] for item in stats["queries"])
    stats["unique_threads"] = len({row["thread_id"] for row in rows})
    stats["rows"] = len(rows)
    return rows, stats


def render_summary(rows: list[dict[str, Any]], stats: dict[str, Any], themes: dict[str, dict[str, Any]]) -> str:
    theme_counts: Counter[str] = Counter()
    theme_examples: dict[str, dict[str, Any]] = {}
    subreddit_counts: Counter[str] = Counter()
    for row in rows:
        subreddit_counts[row["subreddit"]] += 1
        for theme_id in filter(None, row["theme_ids"].split(";")):
            theme_counts[theme_id] += 1
            theme_examples.setdefault(theme_id, row)

    lines = [
        "# Reddit Pain Research Snapshot",
        "",
        f"Generated: `{stats['generated_at_utc']}`",
        "",
        "Refresh command:",
        "",
        "```bash",
        "python3 tools/adoption/reddit_research.py",
        "```",
        "",
        "Collection mode:",
        "",
        "- Discovery uses Reddit HTML search pages instead of JSON search APIs.",
        "- Evidence rows are collected from Reddit comment partials at `/svc/shreddit/comments/...`.",
        "- Every exported row keeps the direct Reddit comment permalink plus the exact fetch URL used to collect it.",
        "- Rows are emitted only when aerospace terms, developer-surface terms, and pain terms all match.",
        "",
        "## Coverage",
        "",
        "| Metric | Value |",
        "| --- | --- |",
        f"| Successful queries | `{len(stats['queries'])}` |",
        f"| Search results inspected | `{stats['searched_threads']}` |",
        f"| Seed threads | `{stats['seed_threads']}` |",
        f"| Threads fetched | `{stats['fetched_threads']}` |",
        f"| Unique threads exported | `{stats['unique_threads']}` |",
        f"| Quote rows exported | `{stats['rows']}` |",
        f"| Skipped query failures | `{len(stats['query_failures'])}` |",
        f"| Skipped fetch failures | `{len(stats['fetch_failures'])}` |",
        "",
        "## Top Subreddits",
        "",
        "| Subreddit | Quote Rows |",
        "| --- | --- |",
    ]
    for subreddit, count in subreddit_counts.most_common(10):
        lines.append(f"| `{subreddit}` | `{count}` |")

    lines.extend(
        [
            "",
            "## Theme Summary",
            "",
            "| Theme | Quote Rows | Suggested Ergonomics Surface | Example |",
            "| --- | --- | --- | --- |",
        ]
    )
    for theme_id, count in theme_counts.most_common():
        theme = themes.get(theme_id, {"label": theme_id, "ergonomics_surface": ""})
        example = theme_examples[theme_id]
        sample = example["quote_text"].replace("|", "\\|")
        if len(sample) > 120:
            sample = sample[:117].rstrip() + "..."
        lines.append(
            f"| `{theme['label']}` | `{count}` | {theme.get('ergonomics_surface', '')} | {sample} |"
        )

    lines.extend(
        [
            "",
            "## Query Inventory",
            "",
            "| Query ID | Subreddit | Query | Search Results |",
            "| --- | --- | --- | --- |",
        ]
    )
    for query in stats["queries"]:
        subreddit = query["subreddit"] or "all"
        lines.append(
            f"| `{query['query_id']}` | `{subreddit}` | `{query['query']}` | `{query['results']}` |"
        )
    return "\n".join(lines) + "\n"


def write_csv(rows: list[dict[str, Any]], path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if not rows:
        path.write_text("", encoding="utf-8")
        return
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)


def write_json(rows: list[dict[str, Any]], stats: dict[str, Any], path: Path) -> None:
    payload = {
        "generated_at_utc": stats["generated_at_utc"],
        "stats": stats,
        "rows": rows,
    }
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Collect Reddit pain-point evidence for avionics and aerospace developer ergonomics.",
    )
    parser.add_argument("--config", type=Path, default=DEFAULT_CONFIG)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    config = load_yaml(args.config)
    rows, stats = collect_rows(config)
    themes = theme_lookup(config)
    output_dir = args.output_dir
    output_dir.mkdir(parents=True, exist_ok=True)
    write_csv(rows, output_dir / CSV_NAME)
    write_json(rows, stats, output_dir / JSON_NAME)
    (output_dir / SUMMARY_NAME).write_text(render_summary(rows, stats, themes), encoding="utf-8")
    print(f"wrote {stats['rows']} rows across {stats['unique_threads']} threads to {output_dir / CSV_NAME}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
