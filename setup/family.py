import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "core" / "data" / "family.json"
FORMAT = "formmcp-family"


def git(repo, *args):
    try:
        return subprocess.check_output(["git", "-C", str(repo), *args], text=True, stderr=subprocess.DEVNULL).strip()
    except (OSError, subprocess.CalledProcessError):
        return ""


def build(compiler):
    apps_dir = compiler / "apps"
    if not apps_dir.is_dir():
        sys.exit(f"family: {apps_dir} is not formcompiler's registry folder")
    version = (compiler / "VERSION").read_text(encoding="utf-8").strip() if (compiler / "VERSION").is_file() else ""
    apps = []
    for path in sorted(apps_dir.glob("*.json")):
        entry = json.loads(path.read_text(encoding="utf-8"))
        catalog = entry["catalog"]
        agents = entry.get("agents") or {"skills": "", "mcp": False}
        apps.append({
            "id": catalog["id"],
            "app": entry["app"],
            "name": catalog["name"],
            "kind": catalog["kind"],
            "summary": catalog["summary"],
            "repository": f"https://github.com/{entry['repository']}",
            "state": catalog["state"],
            "released": bool(entry.get("enabled")),
            "mcp": bool(agents.get("mcp")),
            "skills": bool(agents.get("skills")),
        })
    apps.sort(key=lambda a: a["id"])
    return {
        "format": FORMAT,
        "format_version": 1,
        "source": {"repository": "nodemode-ai/formcompiler", "version": version, "commit": git(compiler, "rev-parse", "HEAD")},
        "apps": apps,
    }


def main():
    parser = argparse.ArgumentParser(description="core/data/family.json from formcompiler's registry")
    parser.add_argument("--compiler", required=True, help="a checkout of nodemode-ai/formcompiler")
    parser.add_argument("--check", action="store_true", help="fail when core/data/family.json differs")
    args = parser.parse_args()
    text = json.dumps(build(Path(args.compiler).resolve()), indent=2, ensure_ascii=False) + "\n"
    current = OUT.read_text(encoding="utf-8") if OUT.is_file() else ""
    if args.check:
        print("family: in step" if current == text else "family: drifted (run python3 setup/family.py --compiler PATH)")
        sys.exit(0 if current == text else 1)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    count = len(json.loads(text)["apps"])
    print(f"family: {OUT.relative_to(ROOT)} {'unchanged' if current == text else 'written'}, {count} apps")


if __name__ == "__main__":
    main()
