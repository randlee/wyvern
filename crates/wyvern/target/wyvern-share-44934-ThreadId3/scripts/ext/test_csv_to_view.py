"""Unit tests for csv_to_view.py — JSON shape and staged-file layout."""
import json
import subprocess
import sys
from pathlib import Path

SCRIPT = Path(__file__).parent / "csv_to_view.py"
SAMPLE_CSV = Path(__file__).parent.parent.parent / "fixtures" / "sample.csv"


def run_script(*args, **kwargs):
    return subprocess.run(
        [sys.executable, str(SCRIPT)] + list(args),
        capture_output=True,
        text=True,
        **kwargs,
    )


def test_html_format_produces_rows_json(tmp_path):
    result = run_script(str(SAMPLE_CSV), "--out", str(tmp_path), "--format", "html")
    assert result.returncode == 0, f"Script failed: {result.stderr}"
    rows_path = tmp_path / "data" / "rows.json"
    assert rows_path.exists(), "data/rows.json not created"
    data = json.loads(rows_path.read_text())
    assert "columns" in data and "rows" in data and "meta" in data
    assert data["columns"] == ["name", "age", "city", "score"]
    assert len(data["rows"]) == 5
    assert data["meta"]["truncated"] is False
    assert data["meta"]["row_count"] == 5


def test_html_format_produces_view_html(tmp_path):
    result = run_script(str(SAMPLE_CSV), "--out", str(tmp_path), "--format", "html")
    assert result.returncode == 0, f"Script failed: {result.stderr}"
    # view.html is copied from share assets
    assert (tmp_path / "pages" / "view.html").exists()


def test_html_format_produces_js_css(tmp_path):
    result = run_script(str(SAMPLE_CSV), "--out", str(tmp_path), "--format", "html")
    assert result.returncode == 0, f"Script failed: {result.stderr}"
    assert (tmp_path / "shared" / "table.js").exists()
    assert (tmp_path / "shared" / "table.css").exists()


def test_markdown_format_outputs_to_stdout(tmp_path):
    result = run_script(str(SAMPLE_CSV), "--out", str(tmp_path), "--format", "markdown")
    assert result.returncode == 0, f"Script failed: {result.stderr}"
    assert "name" in result.stdout
    assert "Alice" in result.stdout
    assert "|" in result.stdout  # pipe table


def test_truncation_at_10000_rows(tmp_path):
    """Script truncates at 10,000 rows."""
    import csv

    big_csv = tmp_path / "big.csv"
    with open(big_csv, "w", newline="") as f:
        w = csv.writer(f)
        w.writerow(["id", "value"])
        for i in range(10_005):
            w.writerow([i, f"val{i}"])
    result = run_script(str(big_csv), "--out", str(tmp_path / "out"), "--format", "html")
    assert result.returncode == 0, f"Script failed: {result.stderr}"
    data = json.loads((tmp_path / "out" / "data" / "rows.json").read_text())
    assert data["meta"]["truncated"] is True
    assert len(data["rows"]) == 10_000
    assert data["meta"]["row_count"] == 10_000


def test_missing_csv_exits_nonzero(tmp_path):
    missing = tmp_path / "no-such.csv"
    result = run_script(str(missing), "--out", str(tmp_path / "out"), "--format", "html")
    assert result.returncode == 1
    assert result.stderr


def test_empty_csv_exits_nonzero(tmp_path):
    empty = tmp_path / "empty.csv"
    empty.write_text("")
    result = run_script(str(empty), "--out", str(tmp_path / "out"), "--format", "html")
    assert result.returncode == 1
    assert result.stderr
