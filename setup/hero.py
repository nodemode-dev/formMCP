import argparse
import io
import sys
from pathlib import Path

from playwright.sync_api import sync_playwright

AGENTS = ["claude", "codex", "cursor", "antigravity"]
TASKS = {
    "claude": "cut a 30 second trailer from these video clips",
    "codex": "convert a folder of MOV files to H.264 MP4",
    "cursor": "retouch a photo and remove the background",
    "antigravity": "clean up podcast audio and master it",
}


def run_tool(page, tool, agent, args):
    console = page.locator("#lab .lab-pane").filter(has=page.locator("select[aria-label='MCP tool']"))
    console.locator("select[aria-label='MCP tool']").select_option(tool)
    console.locator(".agent-pick select").select_option(agent)
    console.locator("textarea.lab-args").fill(args)
    console.get_by_role("button", name="Run").click()
    page.wait_for_timeout(150)


def capture(root, out, scheme):
    page_url = (root / "demo" / "index.html").as_uri()
    with sync_playwright() as p:
        browser = p.chromium.launch()
        context = browser.new_context(viewport={"width": 1600, "height": 1000}, device_scale_factor=2, color_scheme=scheme, timezone_id="America/New_York", locale="en-US")
        page = context.new_page()
        errors = []
        page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.goto(page_url)
        page.wait_for_load_state("load")
        page.wait_for_timeout(1500)
        cards = page.locator(".client[data-client]")
        names = [cards.nth(i).get_attribute("data-client") for i in range(cards.count())]
        if names != AGENTS:
            raise SystemExit(f"the window shows the agent cards {names}, not {AGENTS}")
        for agent in AGENTS:
            page.locator(f'.client[data-client="{agent}"] [data-connect]').click()
            page.wait_for_timeout(150)
            page.locator("[data-lab-open]").click()
            page.locator("#lab .lab-tabs button", has_text="Console").click()
            run_tool(page, "family_apps", agent, '{ "mcp_only": true }')
            run_tool(page, "suggest_app", agent, '{ "task": "%s", "limit": 3 }' % TASKS[agent])
            page.locator("#lab .lab-head button", has_text="Close").click()
            page.wait_for_timeout(150)
        page.mouse.move(0, 0)
        page.evaluate("window.scrollTo(0, 0)")
        page.wait_for_timeout(1500)
        shot = page.screenshot(type="png")
        browser.close()
    if errors:
        raise SystemExit("console errors: " + " | ".join(errors))
    try:
        from PIL import Image
    except ImportError:
        out.write_bytes(shot)
        return None
    image = Image.open(io.BytesIO(shot)).convert("RGB")
    image.save(out, optimize=True)
    return image


def preview(image, path):
    from PIL import Image
    width = 960
    small = image.resize((width, round(image.height * width / image.width)), Image.Resampling.LANCZOS)
    small.save(path, "WEBP", quality=80)


def main():
    parser = argparse.ArgumentParser(description="Remake docs/images/formmcp-hero.png from the built demo 1 (demo/index.html).")
    parser.add_argument("--out", type=Path, help="where to write the picture (default: docs/images/formmcp-hero.png)")
    parser.add_argument("--scheme", choices=["dark", "light"], default="dark")
    parser.add_argument("--preview", type=Path, help="where to write demo 1's 960 px preview (default: demo/assets/media/shots/mcp-hero.webp)")
    parser.add_argument("--preview-only", action="store_true", help="make the preview from the picture already at --out, without taking a new one")
    a = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    out = a.out or root / "docs" / "images" / "formmcp-hero.png"
    small = a.preview or root / "demo" / "assets" / "media" / "shots" / "mcp-hero.webp"
    if a.preview_only:
        from PIL import Image
        image = Image.open(out).convert("RGB")
    else:
        image = capture(root, out, a.scheme)
        print(f"wrote {out}")
    if image is None:
        print("Pillow is not installed: the preview was not made")
        return 1
    preview(image, small)
    print(f"wrote {small}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
