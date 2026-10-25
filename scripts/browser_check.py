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
import base64
import functools
import json
import mimetypes
import os
import re
import sys
import threading
import time
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
    from playwright.sync_api import expect

    # Main-world wait_for_function evaluation can unlock a pending Web Audio
    # resume through CDP's user gesture, invalidating the strict autoplay check.
    canvas = page.locator("canvas#film")
    expect(canvas).to_have_attribute("width", re.compile(r"^[1-9]\d*$"), timeout=30_000)
    expect(canvas).to_have_attribute("height", re.compile(r"^[1-9]\d*$"))
    expect(canvas).to_have_attribute("data-time", re.compile(r"^\d+(?:\.\d+)?$"))


def check_autoplay_gate(page) -> None:
    """Read the untouched document without granting a DevTools user gesture."""
    session = page.context.new_cdp_session(page)
    deadline = time.monotonic() + 30
    try:
        while time.monotonic() < deadline:
            result = session.send("Runtime.evaluate", {
                "expression": """(() => {
                    const canvas = document.querySelector('#film');
                    const gate = document.querySelector('#audio-gate');
                    if (!canvas?.dataset.audioState || !gate) return null;
                    const box = gate.getBoundingClientRect();
                    const style = getComputedStyle(gate);
                    return {state:canvas.dataset.audioState,
                        active:navigator.userActivation.hasBeenActive,
                        nativeStates:window.__aiHistoryAudioProbes.map(p => p.context.state),
                        visible:box.width > 0 && box.height > 0
                            && style.display !== 'none' && style.visibility !== 'hidden'};
                })()""",
                "returnByValue": True,
                "userGesture": False,
            })["result"].get("value")
            if result is not None:
                assert not result["active"], "Autoplay was inspected after user activation"
                assert result["state"] == "suspended", f"Strict autoplay was not blocked: {result}"
                assert result["nativeStates"] and all(
                    state == "suspended" for state in result["nativeStates"]
                ), f"Native audio context escaped the autoplay block: {result}"
                assert result["visible"], "Blocked autoplay has no visible activation gate"
                return
            page.wait_for_timeout(50)
        raise AssertionError("The untouched page did not initialize its autoplay gate")
    finally:
        session.detach()


def check_load_failures(browser, base: str) -> list[str]:
    """Exercise both loader stages, then verify that a refresh can recover."""
    from playwright.sync_api import expect

    checked = []
    for asset in ("pkg/aihistory.js", "pkg/aihistory_bg.wasm"):
        context = browser.new_context()
        pattern = f"**/{asset}*"
        context.route(pattern, lambda route: route.abort())
        page = context.new_page()
        page.goto(base, wait_until="domcontentloaded")
        error = page.locator("#load-error")
        expect(error).to_be_visible()
        expect(error).to_have_attribute("role", "alert")
        expect(error).to_contain_text("Please refresh and try again.")
        expect(page.locator("#film")).to_have_count(0)
        context.unroute(pattern)
        page.reload(wait_until="domcontentloaded")
        wait_for_canvas(page)
        expect(error).to_have_count(0)
        context.close()
        checked.append(asset)
    return checked


def check_playback(page) -> None:
    canvas = page.locator("#film")
    assert page.locator("#sound-toggle").get_attribute("aria-pressed") == "true", "Sound must be enabled by default"
    # The initial gate is exercised explicitly in verify(). After a seek the
    # already-unlocked context resumes asynchronously, so its transient gate
    # must not be mistaken for a second activation request.
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


def set_details_open(page, selector: str, opened: bool) -> None:
    """Use the visible summary control; never force the DOM's open state."""
    from playwright.sync_api import expect

    panel = page.locator(selector)
    if not panel.count():
        return
    assert panel.evaluate("node => node.tagName === 'DETAILS'"), f"Expected a details panel: {selector}"
    if (panel.get_attribute("open") is not None) != opened:
        panel.locator(":scope > summary").click()
    if opened:
        expect(panel).to_have_attribute("open", "")
    else:
        assert panel.get_attribute("open") is None, f"Panel did not close: {selector}"


def reveal_details_ancestors(page, selector: str) -> None:
    """Open outer disclosures before interacting with a control inside them."""
    for panel in page.locator(selector).locator("xpath=ancestor::details[not(@open)]").all():
        panel.locator(":scope > summary").click()


def open_insight(page) -> None:
    set_details_open(page, "#insight-panel", True)


def open_archive(page) -> None:
    from playwright.sync_api import expect

    set_details_open(page, "#archive-panel", True)
    reveal_details_ancestors(page, "#search")
    search = page.locator("#search")
    if not search.is_visible() and page.locator("#archive-toggle").is_visible():
        page.locator("#archive-toggle").click()
    expect(search).to_be_visible()


def choose_history(page, index: int) -> None:
    selector = f'#timeline-list [data-action="seek"][data-index="{index}"]'
    reveal_details_ancestors(page, selector)
    page.locator(selector).click()


def check_stage_layout(page, label: str) -> dict:
    """Measure the actual visible narrative and primary controls, not the archive."""
    from playwright.sync_api import expect

    check_no_overflow(page)
    selectors = ("#film", "#scene-caption", "#scene-summary", "#progress", "#play-toggle", "#sound-toggle")
    for selector in selectors:
        expect(page.locator(selector)).to_be_visible()
    metrics = page.evaluate(
        """() => {
            const read = selector => {
                const node = document.querySelector(selector);
                const r = node.getBoundingClientRect();
                const style = getComputedStyle(node);
                const x = r.left + r.width / 2, y = r.top + r.height / 2;
                const hit = document.elementFromPoint(x, y);
                return {x:r.x, y:r.y, width:r.width, height:r.height,
                    font:Number.parseFloat(style.fontSize), text:node.textContent.trim(),
                    unobstructed:hit !== null && (hit === node || node.contains(hit))};
            };
            return {viewport:{width:innerWidth,height:innerHeight},
                stage:read('#stage-shell'), canvas:read('#film'),
                caption:read('#scene-caption'), summary:read('#scene-summary'),
                play:read('#play-toggle'), sound:read('#sound-toggle'), progress:read('#progress')};
        }"""
    )
    viewport = metrics["viewport"]
    assert metrics["stage"]["width"] >= viewport["width"] * 0.80, f"{label}: the cinematic stage is too narrow"
    for name in ("caption", "summary"):
        item = metrics[name]
        assert item["text"], f"{label}: missing active {name}"
        assert item["font"] >= 14.0, f"{label}: {name} is only {item['font']}px"
        assert item["height"] > 0 and item["x"] >= -1, f"{label}: {name} is clipped"
        assert item["x"] + item["width"] <= viewport["width"] + 1, f"{label}: {name} exceeds viewport width"
    for name in ("play", "sound"):
        item = metrics[name]
        assert item["width"] >= 36 and item["height"] >= 36, f"{label}: {name} target is too small"
        assert item["unobstructed"], f"{label}: {name} is covered or outside the viewport"
    assert metrics["progress"]["unobstructed"], f"{label}: progress is covered or outside the viewport"
    return metrics


def pause_and_check(page) -> None:
    from playwright.sync_api import expect

    canvas = page.locator("#film")
    if canvas.get_attribute("data-playing") == "true":
        page.locator("#play-toggle").click()
    expect(canvas).to_have_attribute("data-playing", "false")
    before = float(canvas.get_attribute("data-time"))
    # Read the backing pixels directly: element screenshots scroll a deliberately
    # cropped mobile canvas into view, changing the screenshot's clipping area.
    page.wait_for_timeout(120)
    before_box = canvas.bounding_box()
    frame = canvas.evaluate("node => node.toDataURL('image/png')")
    # Observe multiple real animation frames while the transport is paused.
    page.wait_for_timeout(350)
    after = float(canvas.get_attribute("data-time"))
    assert abs(after - before) < 0.002, f"Paused transport advanced: {before} -> {after}"
    after_frame = canvas.evaluate("node => node.toDataURL('image/png')")
    if frame != after_frame:
        output = LOCAL / "screenshots"
        output.mkdir(parents=True, exist_ok=True)
        viewport = page.viewport_size
        stem = f"pause-{viewport['width']}x{viewport['height']}-{before:.1f}"
        (output / f"{stem}-before.png").write_bytes(base64.b64decode(frame.split(",", 1)[1]))
        (output / f"{stem}-after.png").write_bytes(base64.b64decode(after_frame.split(",", 1)[1]))
        raise AssertionError(
            f"Paused canvas changed at {before:.3f}s; viewport={viewport}; "
            f"bounds={before_box} -> {canvas.bounding_box()}; frames={stem}"
        )
    after_box = canvas.bounding_box()
    assert before_box and after_box
    assert all(abs(before_box[key] - after_box[key]) < 0.1 for key in before_box), (
        f"Paused canvas moved on screen: {before_box} -> {after_box}"
    )


def check_keyboard_transport(page) -> None:
    """Repeated keydown events from one held key must toggle only once."""
    from playwright.sync_api import expect

    pause_and_check(page)
    page.locator("#film").click(position={"x": 20, "y": 50})
    page.keyboard.down("Space")
    expect(page.locator("#film")).to_have_attribute("data-playing", "true")
    scroll = page.evaluate("scrollY")
    page.keyboard.down("Space")
    expect(page.locator("#film")).to_have_attribute("data-playing", "true")
    page.keyboard.down("Space")
    page.keyboard.up("Space")
    expect(page.locator("#film")).to_have_attribute("data-playing", "true")
    assert page.evaluate("scrollY") == scroll, "Held Space unexpectedly scrolled the page"
    page.keyboard.press("Space")
    expect(page.locator("#film")).to_have_attribute("data-playing", "false")


def check_live_motion_preference(page) -> None:
    from playwright.sync_api import expect

    page.locator("#play-toggle").click()
    expect(page.locator("#film")).to_have_attribute("data-playing", "true")
    page.emulate_media(reduced_motion="reduce")
    expect(page.locator("#motion-toggle")).to_have_attribute("aria-pressed", "true")
    expect(page.locator("#film")).to_have_attribute("data-playing", "false")
    pause_and_check(page)
    page.emulate_media(reduced_motion="no-preference")
    expect(page.locator("#motion-toggle")).to_have_attribute("aria-pressed", "false")
    expect(page.locator("#film")).to_have_attribute("data-playing", "false")
    # Explicit page settings take precedence over later OS preference changes.
    page.locator("#motion-toggle").click()
    page.emulate_media(reduced_motion="reduce")
    page.emulate_media(reduced_motion="no-preference")
    page.wait_for_timeout(150)
    expect(page.locator("#motion-toggle")).to_have_attribute("aria-pressed", "true")
    page.locator("#motion-toggle").click()


def check_canvas_language_neutrality(page) -> None:
    from playwright.sync_api import expect

    pause_and_check(page)
    canvas = page.locator("#film")
    frame = canvas.evaluate("node => node.toDataURL('image/png')")
    time = canvas.get_attribute("data-time")
    language = page.locator("#language")
    original = language.input_value()
    for code in ("en", "ja", "es", "zh"):
        language.select_option(code)
        expect(page.locator("html")).to_have_attribute("lang", re.compile(rf"^{code}(?:-|$)"))
        page.wait_for_timeout(120)
        assert canvas.get_attribute("data-time") == time, "Language changed paused time"
        assert canvas.evaluate("node => node.toDataURL('image/png')") == frame, (
            f"Language {code} changed the language-neutral canvas"
        )
    language.select_option(original)


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
    set_details_open(page, "#archive-panel", False)
    set_details_open(page, "#insight-panel", False)
    page.locator("#experience").scroll_into_view_if_needed()
    check_no_overflow(page)
    page.screenshot(path=str(output / f"{name}.png"), full_page=True)
    page.screenshot(path=str(output / f"{name}-viewport.png"))
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
    layouts: dict[str, dict] = {}
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
            if request.resource_type == "media"
            or Path(urllib.parse.urlsplit(request.url).path).suffix.lower() in media_extensions
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
        load_failures = check_load_failures(browser, base)
        context = browser.new_context(viewport={"width": 1440, "height": 900}, locale="zh-CN")
        context.add_init_script(AUDIO_PROBE_SCRIPT)
        page = context.new_page()
        watch(page)
        response = page.goto(base, wait_until="domcontentloaded")
        assert response and response.status == 200
        # Locator assertions can grant a CDP user gesture before asynchronous
        # initialization finishes. Observe strict autoplay before any of them.
        check_autoplay_gate(page)
        page.locator("#audio-gate").click()
        expect(page.locator("#film")).to_have_attribute("data-audio-state", "running")
        expect(page.locator("#audio-gate")).to_be_hidden()
        language = page.locator("#language")
        expect(language).to_be_visible()
        expect(language).to_have_value("zh")
        timeline = page.locator('#timeline-list [data-action="seek"]')
        expect(timeline).to_have_count(16)
        model_cards = page.locator("#model-grid .model-card")
        expect(model_cards).to_have_count(20)
        wait_for_canvas(page)
        for selector in ("#archive-panel", "#insight-panel"):
            panel = page.locator(selector)
            if panel.count():
                assert panel.get_attribute("open") is None, f"Initial view must keep {selector} folded"
        expect(page.locator("video, audio")).to_have_count(0)
        expect(page.locator("#sound-toggle")).to_have_attribute("aria-pressed", "true")
        check_playback(page)
        page.wait_for_function(
            "() => { const peak = window.__aiHistoryAudioPeak(); "
            "if (peak <= 0.0001) return false; window.__aiHistoryAudiblePeak = peak; return true; }",
            timeout=10_000,
        )
        audible_peak = page.evaluate("window.__aiHistoryAudiblePeak")
        assert 0.0001 < audible_peak <= 1.0, "Synthesized sound is silent or clips"

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
        layouts["desktop-start"] = check_stage_layout(page, "desktop start")
        check_keyboard_transport(page)
        check_live_motion_preference(page)

        # Seek through a real timeline control before changing language. This
        # makes playback-reset bugs observable instead of only checking t=0.
        choose_history(page, 3)
        expect(page.locator("#event-3")).to_have_attribute("aria-current", "step")
        expect(page.locator("#story-number")).to_have_text("04 / 16")
        assert page.locator("#story-source").get_attribute("href").startswith("https://")
        for code in ("en", "ja", "es", "zh"):
            previous = float(page.locator("#film").get_attribute("data-time"))
            old_text = page.locator("#scene-summary").inner_text()
            language.select_option(code)
            expect(language).to_have_value(code)
            expect(page.locator("html")).to_have_attribute("lang", re.compile(rf"^{code}(?:-|$)"))
            expect(page.locator("#scene-summary")).not_to_have_text(old_text)
            wait_for_canvas(page)
            current = float(page.locator("#film").get_attribute("data-time"))
            assert previous - 0.1 <= current <= previous + 3.0, f"Language {code} moved the transport"
            expect(page.locator("#film")).to_have_attribute("data-playing", "true")
            check_playback(page)
            check_no_overflow(page)

        # Space on a disclosure uses its native behaviour, without toggling
        # the film through the document-wide playback shortcut.
        set_details_open(page, "#insight-panel", False)
        summary = page.locator("#insight-panel > summary")
        summary.press("Space")
        expect(page.locator("#insight-panel")).to_have_attribute("open", "")
        expect(page.locator("#film")).to_have_attribute("data-playing", "false")
        summary.press("Space")
        expect(page.locator("#insight-panel")).not_to_have_attribute("open", "")
        expect(page.locator("#film")).to_have_attribute("data-playing", "false")

        # Deep reading pauses the film, giving readers time to inspect sources.
        open_insight(page)
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
        choose_history(page, 12)
        pause_and_check(page)
        expect(page.locator("#event-12")).to_have_attribute("aria-current", "step")
        expect(page.locator("#stage-date")).to_contain_text("1989")
        expect(page.locator("#story-number")).to_have_text("13 / 16")
        screenshot_scene(page, output, "desktop-cnn")
        layouts["desktop-cnn"] = check_stage_layout(page, "desktop CNN")

        # Keep representative artwork available for a human visual review.
        # Functional canvas changes alone cannot establish illustration quality.
        for index, name in (
            (2, "desktop-theatre"),
            (3, "desktop-automata"),
            (4, "desktop-writer"),
            (7, "desktop-tape"),
            (8, "desktop-neuron"),
            (9, "desktop-dartmouth"),
            (11, "desktop-backprop"),
            (13, "desktop-chess"),
            (15, "desktop-transformer"),
        ):
            choose_history(page, index)
            check_playback(page)
            pause_and_check(page)
            screenshot_scene(page, output, name)

        # The atlas filters models, and its cards seek to actual release dates.
        open_archive(page)
        search = page.locator("#search")
        page.locator("#archive-toggle").press("Tab")
        expect(search).to_be_focused()
        assert search.evaluate(
            "node => { const style = getComputedStyle(node); "
            "return node.matches(':focus-visible') && style.outlineStyle !== 'none' "
            "&& Number.parseFloat(style.outlineWidth) >= 2; }"
        ), "Keyboard focus on the archive search has no visible outline"
        search.fill("DeepSeek-R1")
        expect(model_cards).to_have_count(1)
        expect(page.locator("#result-count")).to_have_text("1")
        model_name = model_cards.first.locator("strong").inner_text()
        model_cards.first.click()
        open_insight(page)
        expect(page.locator("#story-title")).to_have_text(model_name)
        expect(page.locator("#film")).to_have_attribute("data-playing", "false")
        expect(page.locator("#story-detail")).to_be_visible()
        expect(page.locator("#event-race")).to_have_attribute("aria-current", "step")
        open_archive(page)
        search.fill("no-matching-model-12345")
        expect(model_cards).to_have_count(0)
        expect(page.locator("#model-grid .empty-results")).to_be_visible()
        search.fill("")
        expect(model_cards).to_have_count(20)

        # A later observed benchmark must not be backdated to a model's release.
        search.fill("Gemini 3.1 Pro Preview")
        expect(model_cards).to_have_count(1)
        model_cards.first.click()
        open_insight(page)
        expect(page.locator("#story-title")).to_have_text("Gemini 3.1 Pro Preview")
        page.locator("#benchmark").select_option("2")
        expect(page.locator("#active-track")).to_contain_text(
            "AdvancedIF · Mercor / single-shot / highest effort"
        )
        expect(page.locator("#active-track")).to_contain_text("2026-10-24")
        expect(page.locator("#leaderboard .rank-row")).to_have_count(0)
        expect(page.locator("#leaderboard .rank-empty")).to_be_visible()
        # An unranked model used to paint a translated label into the canvas.
        check_canvas_language_neutrality(page)
        open_archive(page)
        search.fill("")
        expect(model_cards).to_have_count(20)

        # Inspect the final published comparisons; protocols must never merge.
        duration = seek_final_scene(page)
        open_insight(page)
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
        first_rank = page.locator("#leaderboard .rank-row").first
        first_rank.press("Enter")
        expect(page.locator("#story-title")).to_have_text(names_latest[0])
        expect(first_rank).to_be_focused()
        page.wait_for_timeout(250)
        expect(first_rank).to_be_focused()
        benchmark.select_option("auto")
        expect(benchmark).to_have_value("auto")
        benchmark.select_option("2")
        screenshot_scene(page, output, "desktop-galaxy")
        layouts["desktop-galaxy"] = check_stage_layout(page, "desktop galaxy")

        # Keyboard navigation must reach a visible control without script focus.
        page.locator("#progress").press("Tab")
        page.keyboard.press("Tab")
        assert page.locator(":focus").count() == 1, "Keyboard focus is missing"
        expect(page.locator(":focus")).to_be_visible()
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
        layouts["mobile-start"] = check_stage_layout(mobile, "mobile start")
        choose_history(mobile, 12)
        pause_and_check(mobile)
        expect(mobile.locator("#stage-date")).to_contain_text("1989")
        screenshot_scene(mobile, output, "mobile-cnn")
        layouts["mobile-cnn"] = check_stage_layout(mobile, "mobile CNN")
        for index, name in (
            (7, "mobile-tape"), (9, "mobile-dartmouth"),
            (11, "mobile-backprop"), (13, "mobile-chess"),
        ):
            choose_history(mobile, index)
            pause_and_check(mobile)
            screenshot_scene(mobile, output, name)
        open_insight(mobile)
        mobile.locator("#jump-race").click()
        expect(mobile.locator("#event-race")).to_have_attribute("aria-current", "step")
        seek_final_scene(mobile)
        open_insight(mobile)
        mobile.locator("#benchmark").select_option("2")
        screenshot_scene(mobile, output, "mobile-galaxy")
        layouts["mobile-galaxy"] = check_stage_layout(mobile, "mobile galaxy")
        check_no_overflow(mobile)
        open_archive(mobile)
        mobile.locator("#search").fill("GPT-6.1 Sol")
        expect(mobile.locator("#model-grid .model-card")).to_have_count(1)
        mobile.locator("#model-grid .model-card").click()
        expect(mobile.locator("#story-title")).to_have_text("GPT-6.1 Sol")
        expect(mobile.locator("#film")).to_have_attribute("data-playing", "false")
        set_details_open(mobile, "#archive-panel", False)
        set_details_open(mobile, "#insight-panel", False)
        mobile.set_viewport_size({"width": 360, "height": 800})
        screenshot_scene(mobile, output, "mobile-narrow")
        layouts["mobile-narrow"] = check_stage_layout(mobile, "360px mobile")
        mobile.locator("#text-size").click()
        expect(mobile.locator("#text-size")).to_have_attribute("aria-pressed", "true")
        check_no_overflow(mobile)
        mobile.screenshot(path=str(output / "mobile-large-text.png"), full_page=True)
        mobile.screenshot(path=str(output / "mobile-large-text-viewport.png"))
        # A disclosure must remain keyboard-operable and expose its sources.
        open_insight(mobile)
        mobile.locator("#mode-deep").click()
        expect(mobile.locator("#story-detail")).to_be_visible()
        expect(mobile.locator("#story-source")).to_be_visible()
        check_no_overflow(mobile)
        browser.close()

    assert not failed_requests, "HTTP errors: " + "; ".join(failed_requests)
    assert not errors, "Browser errors: " + "; ".join(errors)
    assert not media_requests, "Encoded media requests are forbidden: " + "; ".join(media_requests)
    (output / "layout-metrics.json").write_text(
        json.dumps(layouts, ensure_ascii=False, indent=2), encoding="utf-8"
    )
    return {
        "timeline_events": 16, "models": 20, "duration_seconds": duration,
        "languages": ["zh", "en", "ja", "es"], "ranking_protocols_checked": 3,
        "audio_peak_unmuted": audible_peak, "muted_audio_clock_advance": muted_clock_advance,
        "reduced_motion_starts_paused": True, "paused_canvas_stays_still": True,
        "load_failure_recovery_checked": load_failures, "search_keyboard_focus_visible": True,
        "held_space_toggles_once": True, "live_motion_preference_checked": True,
        "ranking_keyboard_focus_preserved": True, "canvas_language_neutral": True,
        "layouts_checked": list(layouts), "screenshots": str(output),
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
