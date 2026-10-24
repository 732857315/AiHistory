"""Verify the real built website, including WASM, canvas animation, and audio.

Install: python -m pip install --target .local/python -r scripts/requirements-test.txt
Run:     python scripts/browser_check.py

Uses an installed Chrome/Edge where available; otherwise install Playwright's
Chromium with `python -m playwright install chromium`. Screenshots are saved to
.local/screenshots/. The temporary server deliberately uses /AiHistory/, matching
the GitHub Pages project path. No browser behavior is mocked.
"""

from __future__ import annotations

import argparse
import functools
import json
import mimetypes
import os
import re
import sys
import threading
import urllib.parse
import urllib.request
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LOCAL = ROOT / ".local"
sys.path.insert(0, str(LOCAL / "python"))

# Add a passive, native analysis branch to every connection to the speakers.
# The original destination connection and return value remain untouched; this
# measures real synthesized samples without replacing any browser audio API.
AUDIO_PROBE_SCRIPT = """
(() => {
    const connect = AudioNode.prototype.connect;
    const probes = [];
    window.__aiHistoryAudioProbes = probes;
    AudioNode.prototype.connect = function (...args) {
        const result = Reflect.apply(connect, this, args);
        const destination = args[0];
        if (destination instanceof AudioDestinationNode) {
            const analyser = this.context.createAnalyser();
            analyser.fftSize = 2048;
            analyser.smoothingTimeConstant = 0;
            Reflect.apply(connect, this, [analyser, args[1] ?? 0]);
            probes.push({analyser, context: this.context,
                samples: new Float32Array(analyser.fftSize)});
        }
        return result;
    };
    window.__aiHistoryAudioPeak = () => {
        let peak = 0;
        for (const probe of probes) {
            if (probe.context.state !== 'running') continue;
            probe.analyser.getFloatTimeDomainData(probe.samples);
            for (const sample of probe.samples) peak = Math.max(peak, Math.abs(sample));
        }
        return peak;
    };
})();
"""


class ProjectHandler(SimpleHTTPRequestHandler):
    """Serve dist at the same subpath as GitHub Pages."""

    def do_GET(self) -> None:
        parsed = urllib.parse.urlsplit(self.path)
        if not parsed.path.startswith("/AiHistory/"):
            self.send_error(404)
            return
        self.path = "/" + parsed.path.removeprefix("/AiHistory/")
        if parsed.query:
            self.path += "?" + parsed.query
        super().do_GET()

    def log_message(self, _format: str, *args) -> None:
        pass


def find_browser(explicit: str | None) -> str | None:
    if explicit:
        candidate = Path(explicit)
        if not candidate.is_file():
            raise RuntimeError(f"Browser does not exist: {candidate}")
        return str(candidate)
    candidates = [
        Path(os.environ.get("LOCALAPPDATA", "")) / "Google/Chrome/Application/chrome.exe",
        Path(os.environ.get("PROGRAMFILES", "")) / "Google/Chrome/Application/chrome.exe",
        Path(os.environ.get("PROGRAMFILES(X86)", "")) / "Microsoft/Edge/Application/msedge.exe",
        Path("/usr/bin/google-chrome"),
        Path("/usr/bin/chromium"),
        Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
    ]
    return next((str(path) for path in candidates if path.is_file()), None)


def inspect_assets(base: str) -> None:
    for asset, expected_type in (
        ("", "text/html"),
        ("style.css", "text/css"),
        ("pkg/aihistory.js", "javascript"),
        ("pkg/aihistory_bg.wasm", "application/wasm"),
    ):
        with urllib.request.urlopen(base + asset, timeout=15) as response:
            assert response.status == 200, f"Asset failed: {asset}"
            content_type = response.headers.get("Content-Type", "")
            assert expected_type in content_type, f"Wrong MIME type: {asset}: {content_type}"
            data = response.read(8)
            assert data, f"Empty asset: {asset}"
            if asset.endswith(".wasm"):
                assert data == b"\x00asm\x01\x00\x00\x00", "Invalid WASM file"


def wait_for_canvas(page) -> None:
    page.wait_for_function(
        "() => { const c = document.querySelector('#film'); "
        "return c instanceof HTMLCanvasElement && c.width > 0 && c.height > 0 "
        "&& c.dataset.time !== undefined && Number.isFinite(Number(c.dataset.time)); }",
        timeout=30_000,
    )


def check_playback(page) -> None:
    canvas = page.locator("#film")
    assert page.locator("#sound-toggle").get_attribute("aria-pressed") == "true", "Sound must be enabled by default"
    gate = page.locator("#audio-gate")
    if gate.is_visible():
        gate.click()
    if canvas.get_attribute("data-playing") != "true":
        page.locator("#play-toggle").click()
    before = float(canvas.get_attribute("data-time"))
    frame = canvas.screenshot()
    page.wait_for_function(
        "before => { const c = document.querySelector('#film'); "
        "return c.dataset.playing === 'true' && Number(c.dataset.time) > before + 0.3; }",
        arg=before,
        timeout=15_000,
    )
    assert frame != canvas.screenshot(), "Canvas time advances but its rendered frame is unchanged"


def check_no_overflow(page) -> None:
    assert page.evaluate(
        "document.documentElement.scrollWidth <= window.innerWidth + 1"
    ), "Page has horizontal overflow"


def pause_and_check(page) -> None:
    from playwright.sync_api import expect

    canvas = page.locator("#film")
    if canvas.get_attribute("data-playing") == "true":
        page.locator("#play-toggle").click()
    expect(canvas).to_have_attribute("data-playing", "false")
    before = float(canvas.get_attribute("data-time"))
    # Observe multiple real animation frames while the transport is paused.
    page.wait_for_timeout(350)
    after = float(canvas.get_attribute("data-time"))
    assert abs(after - before) < 0.002, f"Paused transport advanced: {before} -> {after}"


def seek_final_scene(page) -> float:
    """Use real keyboard controls on the range input, ending two seconds early."""
    from playwright.sync_api import expect

    progress = page.locator("#progress")
    duration = float(progress.get_attribute("max"))
    assert abs(duration - 358.0) < 0.001, "Expected the prelude, race and final 24-second snapshot hold"
    progress.press("End")
    for _ in range(20):
        progress.press("ArrowLeft")
    expect(page.locator("#film")).to_have_attribute("data-playing", "false")
    actual = float(page.locator("#film").get_attribute("data-time"))
    assert abs(actual - (duration - 2.0)) < 0.11, "The progress slider did not seek accurately"
    expect(page.locator("#event-race")).to_have_attribute("aria-current", "step")
    return duration


def screenshot_scene(page, output: Path, name: str) -> None:
    page.locator("#experience").scroll_into_view_if_needed()
    check_no_overflow(page)
    page.screenshot(path=str(output / f"{name}.png"), full_page=True)
    page.locator("#stage-shell").screenshot(path=str(output / f"{name}-stage.png"))


def verify(base: str, browser_path: str | None) -> dict:
    try:
        from playwright.sync_api import expect, sync_playwright
    except ImportError as error:
        raise RuntimeError(
            "Install browser checks with: python -m pip install --target .local/python "
            "-r scripts/requirements-test.txt"
        ) from error

    output = LOCAL / "screenshots"
    output.mkdir(parents=True, exist_ok=True)
    errors: list[str] = []
    failed_requests: list[str] = []
    media_requests: list[str] = []
    media_extensions = {".mp4", ".mp3", ".wav", ".webm", ".ogg", ".m4a", ".flac", ".avi", ".mov", ".aac"}
    media_files = [
        str(path.relative_to(ROOT))
        for directory in (ROOT / "web", ROOT / "assets", ROOT / "dist")
        for path in directory.rglob("*")
        if path.suffix.lower() in media_extensions
    ]
    assert not media_files, "Encoded media files are forbidden: " + ", ".join(media_files)

    def watch(page) -> None:
        page.on("pageerror", lambda error: errors.append(str(error)))
        page.on("console", lambda message: errors.append(message.text) if message.type == "error" else None)
        page.on(
            "request",
            lambda request: media_requests.append(request.url)
            if Path(urllib.parse.urlsplit(request.url).path).suffix.lower() in media_extensions
            else None,
        )
        page.on(
            "response",
            lambda response: failed_requests.append(f"{response.status} {response.url}")
            if response.status >= 400
            else None,
        )

    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(
            executable_path=browser_path,
            headless=True,
            args=["--autoplay-policy=document-user-activation-required"],
        )
        context = browser.new_context(viewport={"width": 1440, "height": 1100}, locale="zh-CN")
        context.add_init_script(AUDIO_PROBE_SCRIPT)
        page = context.new_page()
        watch(page)
        response = page.goto(base, wait_until="domcontentloaded")
        assert response and response.status == 200
        language = page.locator("#language")
        expect(language).to_be_visible()
        expect(language).to_have_value("zh")
        timeline = page.locator('#timeline-list [data-action="seek"]')
        expect(timeline).to_have_count(16)
        model_cards = page.locator("#model-grid .model-card")
        expect(model_cards).to_have_count(20)
        wait_for_canvas(page)
        expect(page.locator("video, audio")).to_have_count(0)
        expect(page.locator("#sound-toggle")).to_have_attribute("aria-pressed", "true")
        # A strict autoplay policy must leave a visible, working activation gate.
        expect(page.locator("#film")).to_have_attribute("data-audio-state", "suspended")
        expect(page.locator("#audio-gate")).to_be_visible()
        page.locator("#audio-gate").click()
        expect(page.locator("#film")).to_have_attribute("data-audio-state", "running")
        expect(page.locator("#audio-gate")).to_be_hidden()
        check_playback(page)
        page.wait_for_function(
            "() => { const peak = window.__aiHistoryAudioPeak(); "
            "if (peak <= 0.0001) return false; window.__aiHistoryAudiblePeak = peak; return true; }",
            timeout=10_000,
        )
        audible_peak = page.evaluate("window.__aiHistoryAudiblePeak")

        # Mute intentionally suspends audio processing to save CPU. Check the
        # native context and its frozen clock; an analyser keeps stale samples
        # while suspended, so a made-up zero peak would not measure silence.
        page.locator("#sound-toggle").click()
        expect(page.locator("#sound-toggle")).to_have_attribute("aria-pressed", "false")
        expect(page.locator("#film")).to_have_attribute("data-audio-state", "suspended")
        page.wait_for_function(
            "() => window.__aiHistoryAudioProbes.length > 0 && "
            "window.__aiHistoryAudioProbes.every(p => p.context.state === 'suspended')"
        )
        muted_times = page.evaluate(
            "window.__aiHistoryAudioProbes.map(p => p.context.currentTime)"
        )
        page.wait_for_timeout(300)
        muted_states = page.evaluate(
            "window.__aiHistoryAudioProbes.map(p => "
            "({state: p.context.state, time: p.context.currentTime}))"
        )
        assert len(muted_states) == len(muted_times), "Mute unexpectedly created an audio context"
        assert all(state["state"] == "suspended" for state in muted_states), "Muted audio resumed unexpectedly"
        muted_clock_advance = max(abs(state["time"] - before) for state, before in zip(muted_states, muted_times))
        assert muted_clock_advance == 0.0, f"Muted audio processing continued: {muted_clock_advance}s"
        page.locator("#sound-toggle").click()
        expect(page.locator("#sound-toggle")).to_have_attribute("aria-pressed", "true")
        expect(page.locator("#film")).to_have_attribute("data-audio-state", "running")
        page.wait_for_function("() => window.__aiHistoryAudioPeak() > 0.0001", timeout=10_000)
        pause_and_check(page)
        page.locator("#progress").press("Home")
        expect(page.locator("#event-0")).to_have_attribute("aria-current", "step")
        assert float(page.locator("#film").get_attribute("data-time")) == 0.0
        screenshot_scene(page, output, "desktop-start")

        # Seek through a real timeline control before changing language. This
        # makes playback-reset bugs observable instead of only checking t=0.
        timeline.nth(3).click()
        expect(page.locator("#event-3")).to_have_attribute("aria-current", "step")
        expect(page.locator("#story-number")).to_have_text("04 / 16")
        assert page.locator("#story-source").get_attribute("href").startswith("https://")
        for code in ("en", "ja", "es", "zh"):
            previous = float(page.locator("#film").get_attribute("data-time"))
            old_text = page.locator("#hero-title").inner_text()
            language.select_option(code)
            expect(language).to_have_value(code)
            expect(page.locator("html")).to_have_attribute("lang", re.compile(rf"^{code}(?:-|$)"))
            expect(page.locator("#hero-title")).not_to_have_text(old_text)
            wait_for_canvas(page)
            current = float(page.locator("#film").get_attribute("data-time"))
            assert previous - 0.1 <= current <= previous + 3.0, f"Language {code} moved the transport"
            expect(page.locator("#film")).to_have_attribute("data-playing", "true")
            check_playback(page)

        # Deep reading pauses the film, giving readers time to inspect sources.
        page.locator("#mode-deep").click()
        expect(page.locator("#mode-deep")).to_have_attribute("aria-pressed", "true")
        expect(page.locator("#story-detail")).to_be_visible()
        pause_and_check(page)
        page.locator("#mode-easy").click()
        expect(page.locator("#mode-easy")).to_have_attribute("aria-pressed", "true")
        expect(page.locator("#story-detail")).to_be_hidden()
        page.locator("#text-size").click()
        expect(page.locator("#text-size")).to_have_attribute("aria-pressed", "true")
        check_no_overflow(page)
        page.locator("#text-size").click()

        # The early CNN scene deliberately receives more screen time.
        timeline.nth(12).click()
        pause_and_check(page)
        expect(page.locator("#event-12")).to_have_attribute("aria-current", "step")
        expect(page.locator("#stage-date")).to_contain_text("1989")
        expect(page.locator("#story-number")).to_have_text("13 / 16")
        screenshot_scene(page, output, "desktop-cnn")

        # The atlas filters models, and its cards seek to actual release dates.
        search = page.locator("#search")
        search.fill("DeepSeek-R1")
        expect(model_cards).to_have_count(1)
        expect(page.locator("#result-count")).to_have_text("1")
        model_name = model_cards.first.locator("strong").inner_text()
        model_cards.first.click()
        expect(page.locator("#story-title")).to_have_text(model_name)
        expect(page.locator("#film")).to_have_attribute("data-playing", "false")
        expect(page.locator("#story-detail")).to_be_visible()
        expect(page.locator("#event-race")).to_have_attribute("aria-current", "step")
        search.fill("no-matching-model-12345")
        expect(model_cards).to_have_count(0)
        expect(page.locator("#model-grid .empty-results")).to_be_visible()
        search.fill("")
        expect(model_cards).to_have_count(20)

        # A later observed benchmark must not be backdated to a model's release.
        search.fill("Gemini 3.1 Pro Preview")
        expect(model_cards).to_have_count(1)
        model_cards.first.click()
        expect(page.locator("#story-title")).to_have_text("Gemini 3.1 Pro Preview")
        page.locator("#benchmark").select_option("2")
        expect(page.locator("#active-track")).to_contain_text(
            "AdvancedIF · Mercor / single-shot / highest effort"
        )
        expect(page.locator("#active-track")).to_contain_text("2026-10-24")
        expect(page.locator("#leaderboard .rank-row")).to_have_count(0)
        expect(page.locator("#leaderboard .rank-empty")).to_be_visible()
        search.fill("")
        expect(model_cards).to_have_count(20)

        # Inspect the final published comparisons; protocols must never merge.
        duration = seek_final_scene(page)
        benchmark = page.locator("#benchmark")
        benchmark.select_option("0")
        expect(page.locator("#active-track")).to_have_text("MMLU · 5-shot / no-CoT")
        names_five = page.locator("#leaderboard .rank-name").all_text_contents()
        scores_five = page.locator("#leaderboard .rank-row strong").all_text_contents()
        assert names_five == [
            "Claude 3.5 Sonnet · June", "Llama 3.1 405B Instruct", "Claude 3 Opus",
            "GPT-4 · March 2023", "Llama 3 70B Instruct",
        ], f"Unexpected five-example ranking: {names_five}"
        assert scores_five == ["88.7", "87.3", "86.8", "86.4", "82.0"], scores_five
        benchmark.select_option("1")
        expect(page.locator("#active-track")).to_have_text("MMLU · 0-shot CoT / simple-evals")
        names_zero = page.locator("#leaderboard .rank-name").all_text_contents()
        scores_zero = page.locator("#leaderboard .rank-row strong").all_text_contents()
        assert names_zero == ["DeepSeek-R1 · 0120", "DeepSeek-V3 · 1226", "GPT-4o · 0513", "Qwen2.5 72B Instruct"], names_zero
        assert scores_zero == ["90.8", "88.5", "87.2", "85.3"], scores_zero
        assert not set(names_zero) & set(names_five), "Benchmark protocols were merged"
        benchmark.select_option("2")
        expect(page.locator("#active-track")).to_contain_text(
            "AdvancedIF · Mercor / single-shot / highest effort"
        )
        expect(page.locator("#active-track")).to_contain_text("2026-10-24")
        names_latest = page.locator("#leaderboard .rank-name").all_text_contents()
        scores_latest = page.locator("#leaderboard .rank-row strong").all_text_contents()
        assert names_latest == ["Gemini 3.1 Pro Preview", "Claude Opus 5.5", "GPT-6.1 Sol"], names_latest
        assert scores_latest == ["86.7", "82.8", "82.4"], scores_latest
        assert not set(names_latest) & set(names_zero + names_five), "Different benchmarks were merged"
        benchmark.select_option("auto")
        expect(benchmark).to_have_value("auto")
        benchmark.select_option("2")
        screenshot_scene(page, output, "desktop-galaxy")

        # Keyboard navigation must reach a visible control without script focus.
        page.locator("#progress").press("Tab")
        page.keyboard.press("Tab")
        assert page.locator(":focus").count() == 1, "Keyboard focus is missing"
        check_no_overflow(page)

        mobile_context = browser.new_context(
            viewport={"width": 390, "height": 844}, locale="zh-CN", is_mobile=True, has_touch=True,
            reduced_motion="reduce",
        )
        mobile = mobile_context.new_page()
        watch(mobile)
        mobile.goto(base, wait_until="domcontentloaded")
        expect(mobile.locator("#language")).to_be_visible()
        wait_for_canvas(mobile)
        # A reduced-motion preference starts paused; animation is opt-in.
        expect(mobile.locator("#motion-toggle")).to_have_attribute("aria-pressed", "true")
        expect(mobile.locator("#film")).to_have_attribute("data-playing", "false")
        assert float(mobile.locator("#film").get_attribute("data-time")) == 0.0
        pause_and_check(mobile)
        screenshot_scene(mobile, output, "mobile-start")
        mobile.locator('#timeline-list [data-action="seek"][data-index="12"]').click()
        pause_and_check(mobile)
        expect(mobile.locator("#stage-date")).to_contain_text("1989")
        screenshot_scene(mobile, output, "mobile-cnn")
        mobile.locator("#jump-race").click()
        expect(mobile.locator("#event-race")).to_have_attribute("aria-current", "step")
        seek_final_scene(mobile)
        mobile.locator("#benchmark").select_option("2")
        screenshot_scene(mobile, output, "mobile-galaxy")
        check_no_overflow(mobile)
        mobile.locator("#text-size").click()
        expect(mobile.locator("#text-size")).to_have_attribute("aria-pressed", "true")
        check_no_overflow(mobile)
        mobile.screenshot(path=str(output / "mobile-large-text.png"), full_page=True)
        browser.close()

    assert not failed_requests, "HTTP errors: " + "; ".join(failed_requests)
    assert not errors, "Browser errors: " + "; ".join(errors)
    assert not media_requests, "Encoded media requests are forbidden: " + "; ".join(media_requests)
    return {
        "timeline_events": 16, "models": 20, "duration_seconds": duration,
        "languages": ["zh", "en", "ja", "es"], "ranking_protocols_checked": 3,
        "audio_peak_unmuted": audible_peak, "muted_audio_clock_advance": muted_clock_advance,
        "reduced_motion_starts_paused": True, "screenshots": str(output),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--browser", help="Optional Chrome/Chromium/Edge executable path")
    args = parser.parse_args()
    dist = ROOT / "dist"
    if not (dist / "index.html").is_file():
        print("Build the website first: python scripts/build.py", file=sys.stderr)
        return 1
    mimetypes.add_type("application/wasm", ".wasm")
    mimetypes.add_type("text/javascript", ".js")
    handler = functools.partial(ProjectHandler, directory=str(dist))
    with ThreadingHTTPServer(("127.0.0.1", 0), handler) as server:
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        base = f"http://127.0.0.1:{server.server_port}/AiHistory/"
        try:
            inspect_assets(base)
            result = verify(base, find_browser(args.browser))
            print("Browser verification passed: " + json.dumps(result, ensure_ascii=False))
            return 0
        except Exception as error:
            print(f"Browser verification failed: {error}", file=sys.stderr)
            return 1
        finally:
            server.shutdown()
            thread.join(timeout=5)


if __name__ == "__main__":
    raise SystemExit(main())
