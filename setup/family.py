import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "core" / "data" / "family.json"
FORMAT = "formmcp-family"
SPEC_FORMAT = "formai-family"
SELF = "formMCP"
USAGE_ERROR = 2


class SpecError(Exception):
    pass


def build(spec):
    if not isinstance(spec, dict) or spec.get("format") != SPEC_FORMAT or spec.get("format_version") != 1:
        raise SpecError(f"not a {SPEC_FORMAT} version 1 document")
    source = spec.get("source")
    if not isinstance(source, dict) or not source.get("repository") or not source.get("registry_version"):
        raise SpecError("the spec has no source.repository and source.registry_version")
    entries = spec.get("apps")
    if not isinstance(entries, list):
        raise SpecError("the spec has no apps list")
    apps = []
    for entry in entries:
        if not isinstance(entry, dict) or entry.get("public") is not True:
            continue
        if entry["app"] == SELF:
            continue
        agents = entry.get("agents") or {}
        apps.append({
            "id": entry["id"],
            "app": entry["app"],
            "name": entry["name"],
            "kind": entry["kind"],
            "summary": entry["summary"],
            "repository": f"https://github.com/{entry['repository']}",
            "launcher": entry["launcher"],
            "state": entry["state"],
            "released": entry.get("release") is not None,
            "mcp": bool(agents.get("mcp")),
        })
    apps.sort(key=lambda a: a["id"])
    return {
        "format": FORMAT,
        "format_version": 1,
        "source": {"repository": source["repository"], "version": source["registry_version"]},
        "apps": apps,
    }


def main():
    parser = argparse.ArgumentParser(description="core/data/family.json from the family spec (family.json built by formadaptor)")
    parser.add_argument("--family", required=True, help="the family spec, a family.json from nodemode-dev/formadaptor")
    parser.add_argument("--check", action="store_true", help="write nothing; exit 1 naming the file when core/data/family.json differs")
    args = parser.parse_args()
    try:
        spec = json.loads(Path(args.family).read_text(encoding="utf-8"))
        text = json.dumps(build(spec), indent=2, ensure_ascii=False) + "\n"
    except (OSError, ValueError, KeyError, TypeError, SpecError) as error:
        print(f"family: {args.family} cannot be used: {error}", file=sys.stderr)
        sys.exit(USAGE_ERROR)
    current = OUT.read_text(encoding="utf-8") if OUT.is_file() else ""
    if args.check:
        name = OUT.relative_to(ROOT).as_posix()
        print(f"family: {name} is in step" if current == text else f"family: {name} differs (run python3 -I setup/family.py --family PATH)")
        sys.exit(0 if current == text else 1)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(text, encoding="utf-8")
    count = len(json.loads(text)["apps"])
    print(f"family: {OUT.relative_to(ROOT)} {'unchanged' if current == text else 'written'}, {count} apps")


if __name__ == "__main__":
    main()
