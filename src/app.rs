use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlInputElement,
    HtmlSelectElement, KeyboardEvent,
};

use crate::{audio::Music, history::HISTORY, models::MODELS, scene, *};

const TRACKS: [(&str, &str); 3] = [
    ("MMLU", "5-shot / no-CoT"),
    ("MMLU", "0-shot CoT / simple-evals"),
    ("AdvancedIF", "Mercor / single-shot / highest effort"),
];
type Shared = Rc<RefCell<State>>;
type FrameCallback = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;

struct State {
    lang: usize,
    time: f64,
    playing: bool,
    speed: f64,
    sound: bool,
    narration: bool,
    deep: bool,
    large: bool,
    reduced: bool,
    track: Option<usize>,
    query: String,
    selected_model: Option<usize>,
    music: Option<Music>,
    music_time: f64,
    last_frame: f64,
    last_dom: f64,
    signature: String,
    draw_signature: String,
    needs_snap: bool,
    active_track: usize,
    positions: Vec<f64>,
    appearances: Vec<f64>,
    speak_key: String,
    speech: Option<web_sys::SpeechSynthesisUtterance>,
}

fn tr(lang: usize, text: [&'static str; 4]) -> &'static str {
    text[lang]
}
fn window() -> web_sys::Window {
    web_sys::window().expect("browser window")
}
fn document() -> Document {
    window().document().expect("browser document")
}
fn element(id: &str) -> Element {
    document()
        .get_element_by_id(id)
        .expect("application element")
}
fn text(id: &str, value: &str) {
    element(id).set_text_content(Some(value));
}
fn attr(id: &str, name: &str, value: &str) {
    let _ = element(id).set_attribute(name, value);
}
fn pressed(id: &str, yes: bool) {
    attr(id, "aria-pressed", if yes { "true" } else { "false" });
}
fn hidden(id: &str, yes: bool) {
    if yes {
        let _ = element(id).set_attribute("hidden", "");
    } else {
        let _ = element(id).remove_attribute("hidden");
    }
}
fn preference(key: &str) -> Option<String> {
    window()
        .local_storage()
        .ok()
        .flatten()
        .and_then(|s| s.get_item(key).ok().flatten())
}
fn save(key: &str, value: &str) {
    if let Some(s) = window().local_storage().ok().flatten() {
        let _ = s.set_item(key, value);
    }
}
fn clock_label(time: f64) -> String {
    let t = time.max(0.0) as u64;
    format!("{}:{:02}", t / 60, t % 60)
}
fn track_index(state: &State) -> usize {
    state.track.unwrap_or_else(|| {
        if race_date(state.time) >= date_value("2026-10-24")
            && MODELS.iter().any(|m| {
                m.protocol == TRACKS[2].1
                    && m.score.is_some()
                    && m.score_observed
                    && date_value(m.reported) <= race_date(state.time)
            })
        {
            2
        } else if MODELS.iter().any(|m| {
            m.protocol == TRACKS[1].1
                && m.score.is_some()
                && date_value(m.reported) <= race_date(state.time)
        }) {
            1
        } else {
            0
        }
    })
}
fn current_model(state: &State) -> Option<usize> {
    state.selected_model.or_else(|| {
        if state.time >= history_duration() + RACE_SECONDS {
            let (metric, protocol) = TRACKS[track_index(state)];
            if let Some(index) = ranking(state.time, metric, protocol).first() {
                return Some(*index);
            }
        }
        MODELS
            .iter()
            .enumerate()
            .filter(|(_, m)| date_value(m.released) <= race_date(state.time))
            .map(|(i, _)| i)
            .next_back()
    })
}

fn coverage_note(lang: usize) -> String {
    let scored: Vec<_> = MODELS.iter().filter(|m| m.score.is_some()).collect();
    let published = scored.iter().filter(|m| !m.score_observed).count();
    let observed = scored.iter().filter(|m| m.score_observed).count();
    let published_date = scored
        .iter()
        .filter(|m| !m.score_observed)
        .map(|m| m.reported)
        .max()
        .unwrap_or("—");
    let observed_date = scored
        .iter()
        .filter(|m| m.score_observed)
        .map(|m| m.reported)
        .max()
        .unwrap_or("—");
    match lang {
        1 => format!(
            "Releases verified 2026-10-24. {published} historical published results through {published_date} + {observed} observed snapshots dated {observed_date}. Snapshots enter only from their observation date; no historical backfill. A curated record, not a global leaderboard."
        ),
        2 => format!(
            "モデル公開日の確認：2026-10-24。公開成績{published}件（最終{published_date}）＋参照スナップショット{observed}件（{observed_date}）。スナップショットは参照日から表示し、過去に遡って使いません。資料の選集であり、世界総合ランキングではありません。"
        ),
        3 => format!(
            "Lanzamientos verificados el 2026-10-24. {published} resultados históricos publicados hasta {published_date} + {observed} instantáneas consultadas el {observed_date}. Las instantáneas solo entran desde su consulta, sin aplicarse al pasado. Una selección, no una clasificación mundial."
        ),
        _ => format!(
            "模型发布核验：2026-10-24。{published} 项历史公布成绩（截止 {published_date}）＋{observed} 项查阅快照（{observed_date}）。快照只从查阅日起展示，不回填历史；这是资料选集，不是全球总榜。"
        ),
    }
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let lang = language_index(
        &preference("aihistory.lang")
            .or_else(|| window().navigator().language())
            .unwrap_or_else(|| "zh".into()),
    );
    let reduced = window()
        .match_media("(prefers-reduced-motion: reduce)")?
        .is_some_and(|m| m.matches());
    let state = Rc::new(RefCell::new(State {
        lang,
        time: 0.0,
        playing: !reduced,
        speed: 1.0,
        sound: true,
        narration: false,
        deep: false,
        large: preference("aihistory.large").is_some_and(|v| v == "true"),
        reduced,
        track: None,
        query: String::new(),
        selected_model: None,
        music: Music::new().ok(),
        music_time: 0.0,
        last_frame: 0.0,
        last_dom: -1000.0,
        signature: String::new(),
        draw_signature: String::new(),
        needs_snap: true,
        active_track: 0,
        positions: vec![0.0; MODELS.len()],
        appearances: vec![0.0; MODELS.len()],
        speak_key: String::new(),
        speech: None,
    }));
    element("app").set_inner_html(r##"
      <a class="skip-link" href="#experience" id="skip-label"></a>
      <div class="page-wrap" id="top">
        <header class="site-header">
          <a class="brand" href="#top" aria-label="AiHistory"><span class="brand-symbol">✳</span><span>AI<span class="brand-light">HISTORY</span><small id="brand-subtitle"></small></span></a>
          <nav class="header-nav" aria-label="Navigation"><a href="#timeline" id="nav-timeline"></a><a href="#archive-panel" data-action="archive" id="nav-atlas"></a></nav>
          <div class="header-tools"><button id="text-size" data-action="large" class="icon-button" aria-pressed="false">A<span class="small-plus">+</span></button><label class="language-wrap"><span aria-hidden="true">◎</span><select id="language" aria-label="Language"><option value="zh">中文</option><option value="en">English</option><option value="ja">日本語</option><option value="es">Español</option></select></label><a class="github-link" href="https://github.com/732857315/AiHistory" target="_blank" rel="noopener noreferrer" aria-label="GitHub">GitHub ↗</a></div>
        </header>
        <main>
          <section class="intro"><h1 id="hero-title"></h1><p id="hero-description"></p><span class="journey-meta" id="journey-meta"></span><span class="sr-only" id="eyebrow"></span></section>
          <section id="experience" class="experience" aria-label="Interactive history">
            <div class="stage-column"><div id="stage-shell" class="stage-shell"><div class="stage-strip"><span><span class="live-dot"></span><span id="stage-label"></span></span><span id="stage-date"></span></div>
              <div class="canvas-wrap"><canvas id="film" width="1200" height="675" role="img" aria-label="Animated history" data-time="0" data-playing="false"></canvas><div class="scene-copy"><p id="scene-kicker"></p><h2 id="scene-caption"></h2><p id="scene-summary"></p><button id="scene-explore" class="scene-explore" data-action="inspect"></button></div><div class="scene-foot"><span class="scene-index" id="scene-index"></span><span class="scene-ability" id="scene-ability"></span></div><div class="race-hud" id="race-hud" hidden><span id="race-hud-label"></span><strong id="race-hud-track"></strong><button data-action="inspect" id="race-hud-link"></button></div><button id="audio-gate" class="audio-gate" data-action="unlock"><span aria-hidden="true">♫</span> <span id="audio-gate-label"></span><span aria-hidden="true">↗</span></button></div>
              <div class="transport"><div class="scrubber"><label for="progress" class="sr-only" id="progress-label"></label><input type="range" id="progress" min="0" max="360" step="0.1" value="0"><span id="time-display">0:00</span></div><div class="transport-row"><div class="transport-buttons"><button id="play-toggle" data-action="play" class="play-button"></button><button data-action="restart" id="restart" class="transport-icon">↺</button><button data-action="sound" id="sound-toggle" class="transport-icon" aria-pressed="true">♫</button><button data-action="narration" id="narration-toggle" class="voice-button" aria-pressed="false"></button></div><div class="transport-options"><label><span class="sr-only" id="speed-label"></span><select id="speed" aria-label="Speed"><option value="0.65">0.65×</option><option value="1" selected>1×</option><option value="1.5">1.5×</option><option value="2">2×</option></select></label><button data-action="motion" id="motion-toggle" class="transport-icon" aria-pressed="false">◌</button><button data-action="fullscreen" id="fullscreen" class="transport-icon">⛶</button></div></div></div>
            </div><p id="transport-note" class="transport-note"></p></div>
          </section>
          <section class="timeline-section" id="timeline"><div class="section-heading"><div><p class="eyebrow" id="timeline-eyebrow"></p><h2 id="timeline-title"></h2></div><p id="timeline-description"></p></div><div class="timeline-track" id="timeline-list"></div></section>
          <details class="insight-section" id="insight-panel"><summary><span id="insight-label"></span><span class="details-hint" id="insight-hint"></span><span class="expand-symbol" aria-hidden="true">+</span></summary>
            <aside class="story-panel" id="story-panel"><div class="story-reading"><div class="panel-top"><span class="eyebrow" id="story-label"></span><span class="story-number" id="story-number"></span></div><h2 id="story-title"></h2><p id="story-summary" class="story-summary"></p><div class="reading-switch"><button id="mode-easy" data-action="easy" aria-pressed="true"></button><button id="mode-deep" data-action="deep" aria-pressed="false"></button></div><div id="story-detail"></div><a id="story-source" class="source-link" target="_blank" rel="noopener noreferrer"></a><button class="jump-button" id="jump-race" data-action="race"></button></div><div class="story-evidence"><div id="capability-box" class="capability-box"><span id="capability-label" class="micro-label"></span><p id="capability-text"></p><div class="capability-rule"></div><span id="limit-label" class="micro-label"></span><p id="limit-text"></p></div><div id="ranking-box" hidden><div class="ranking-header"><label for="benchmark" id="benchmark-label"></label><select id="benchmark"><option value="auto">Auto</option><option value="0">MMLU · 5-shot</option><option value="1">MMLU · CoT</option><option value="2">AdvancedIF / 2026 snapshot</option></select></div><p id="active-track" class="track-note"></p><div id="leaderboard"></div><p id="rank-note" class="rank-note"></p></div></div></aside>
          </details>
          <details class="archive-section" id="archive-panel"><summary id="archive-toggle"><span id="archive-label"></span><span class="details-hint" id="archive-hint"></span><span class="expand-symbol" aria-hidden="true">+</span></summary>
          <section class="atlas-section" id="atlas"><div class="section-heading"><div><p class="eyebrow" id="atlas-eyebrow"></p><h2 id="atlas-title"></h2></div><label class="search-wrap"><span aria-hidden="true">⌕</span><input id="search" type="search" autocomplete="off"><span id="result-count"></span></label></div><p class="atlas-description" id="atlas-description"></p><div class="model-grid" id="model-grid"></div><details class="methodology" id="methodology"><summary id="methodology-label"></summary><p id="methodology-text"></p><p id="methodology-score"></p><a href="https://github.com/732857315/AiHistory/tree/main/sources" target="_blank" rel="noopener noreferrer" id="methodology-source"></a></details></section></details>
        </main><footer><a class="footer-brand" href="https://risc.ink/">risc.ink ↗</a><p id="footer-text"></p><a href="https://github.com/732857315/AiHistory/blob/main/LICENSE" target="_blank" rel="noopener noreferrer">MIT · 732857315</a></footer>
      </div>"##);
    let hash = window().location().hash().unwrap_or_default();
    if let Some(value) = hash
        .strip_prefix("#t=")
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| v.is_finite())
    {
        state.borrow_mut().time = value.clamp(0.0, total_duration());
    }
    localize(&state.borrow());
    bind_events(&state)?;
    let canvas: HtmlCanvasElement = element("film").dyn_into()?;
    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("Canvas unavailable"))?
        .dyn_into()?;
    start_frames(state.clone(), context)?;
    if !reduced {
        resume_audio(&state);
    }
    Ok(())
}

fn localize(state: &State) {
    let l = state.lang;
    let strings = [
        (
            "scene-explore",
            [
                "这个瞬间，改变了什么",
                "Discover what changed",
                "何が変わったのか",
                "Descubre qué cambió",
            ],
        ),
        (
            "insight-label",
            [
                "走近这一刻",
                "Inside this moment",
                "この瞬間を知る",
                "Dentro de este momento",
            ],
        ),
        (
            "insight-hint",
            [
                "能力、原理与历史出处",
                "Ideas, abilities & sources",
                "仕組み・能力・出典",
                "Ideas, capacidades y fuentes",
            ],
        ),
        (
            "archive-label",
            [
                "打开模型档案",
                "Open the model archive",
                "モデル資料を開く",
                "Abrir el archivo de modelos",
            ],
        ),
        (
            "archive-hint",
            [
                "20 个模型 · 按登场时间探索",
                "20 models · explore their arrivals",
                "20モデル・登場の時点へ",
                "20 modelos · sus primeras apariciones",
            ],
        ),
        (
            "race-hud-label",
            [
                "同一指标 · 局部放大比较",
                "SAME BENCHMARK · FOCUSED COMPARISON",
                "同じ指標・拡大比較",
                "MISMO INDICADOR · VISTA AMPLIADA",
            ],
        ),
        (
            "race-hud-link",
            [
                "查看成绩与依据 ↗",
                "Results & sources ↗",
                "成績と出典 ↗",
                "Resultados y fuentes ↗",
            ],
        ),
        (
            "skip-label",
            [
                "跳到动画",
                "Skip to the experience",
                "アニメーションへ",
                "Ir a la experiencia",
            ],
        ),
        (
            "brand-subtitle",
            [
                "人类好奇心的漫长旅程",
                "A history of human curiosity",
                "好奇心がつないだ歴史",
                "Una historia de curiosidad",
            ],
        ),
        (
            "nav-timeline",
            ["时间足迹", "The journey", "時の足跡", "El recorrido"],
        ),
        (
            "nav-atlas",
            ["模型星图", "Model atlas", "モデル星図", "Atlas de modelos"],
        ),
        (
            "eyebrow",
            [
                "从工具到智能 · 一段可以探索的历史",
                "FROM TOOLS TO INTELLIGENCE · AN EXPLORABLE HISTORY",
                "道具から知能へ · 探索できる歴史",
                "DE LAS HERRAMIENTAS A LA INTELIGENCIA",
            ],
        ),
        (
            "hero-title",
            [
                "从第一件工具，到智能的星河。",
                "From the first tool to a galaxy of minds.",
                "最初の道具から、知能の星河へ。",
                "De la primera herramienta a una galaxia de mentes.",
            ],
        ),
        (
            "hero-description",
            [
                "从石器的一道痕迹，到模型间的星河追逐。看见每一次「原来还能这样」。",
                "From a mark in stone to a galaxy of competing minds. Explore the moments that made new things possible.",
                "石に刻まれた跡から、モデルが競う星河まで。「こんなこともできる」をたどる旅。",
                "De una marca en piedra a una galaxia de modelos. Descubre los momentos que abrieron nuevas posibilidades.",
            ],
        ),
        (
            "stage-label",
            [
                "正在穿越历史",
                "HISTORY IN MOTION",
                "歴史を旅する",
                "LA HISTORIA EN MOVIMIENTO",
            ],
        ),
        (
            "audio-gate-label",
            [
                "点击开启声音",
                "Enable sound",
                "音をオンに",
                "Activar sonido",
            ],
        ),
        (
            "progress-label",
            [
                "观看进度",
                "Journey progress",
                "再生位置",
                "Progreso del recorrido",
            ],
        ),
        (
            "speed-label",
            ["播放速度", "Playback speed", "再生速度", "Velocidad"],
        ),
        (
            "story-label",
            [
                "此刻，能力发生改变",
                "A NEW POSSIBILITY",
                "新しい可能性",
                "UNA NUEVA POSIBILIDAD",
            ],
        ),
        (
            "mode-easy",
            ["看故事", "The story", "ストーリー", "La historia"],
        ),
        (
            "mode-deep",
            ["看原理", "Go deeper", "詳しく見る", "Profundizar"],
        ),
        (
            "capability-label",
            [
                "它能做到",
                "WHAT IT MADE POSSIBLE",
                "できること",
                "LO QUE HIZO POSIBLE",
            ],
        ),
        (
            "limit-label",
            [
                "当时的边界",
                "THE LIMIT AT THE TIME",
                "当時の限界",
                "EL LÍMITE DE SU ÉPOCA",
            ],
        ),
        (
            "benchmark-label",
            ["当前赛道", "Current track", "比較する指標", "Comparación"],
        ),
        (
            "jump-race",
            [
                "去看看模型星河 ↗",
                "Explore the model galaxy ↗",
                "モデルの星河へ ↗",
                "Explorar la galaxia ↗",
            ],
        ),
        (
            "timeline-eyebrow",
            [
                "每一道足迹，都通向新的可能",
                "ONE POSSIBILITY LEADS TO ANOTHER",
                "可能性が次の可能性へ",
                "UNA POSIBILIDAD LLEVA A OTRA",
            ],
        ),
        (
            "timeline-title",
            [
                "把时间，握在手里。",
                "Time, at your fingertips.",
                "時間を、その手に。",
                "El tiempo, en tus manos.",
            ],
        ),
        (
            "timeline-description",
            [
                "点击一个年代，回到能力改变的瞬间。",
                "Choose a moment. See what changed.",
                "時代を選び、変化の瞬間へ。",
                "Elige un momento. Descubre qué cambió.",
            ],
        ),
        (
            "atlas-eyebrow",
            [
                "星河里的每一位探索者",
                "MEET THE EXPLORERS",
                "星河を進むモデルたち",
                "CONOCE A LOS EXPLORADORES",
            ],
        ),
        (
            "atlas-title",
            [
                "模型星图",
                "The model atlas",
                "モデル星図",
                "Atlas de modelos",
            ],
        ),
        (
            "atlas-description",
            [
                "按发布时间整理的精选模型。选择模型，回到它登场的时刻。",
                "Selected models, in release order. Select a model to visit its arrival and explore its sources.",
                "公開順に並べた主要モデル。カードを選ぶと登場時点へ移動し、成績と出典を確認できます。",
                "Modelos seleccionados por fecha de lanzamiento. Elige una tarjeta para viajar a su llegada y consultar resultados y fuentes.",
            ],
        ),
        (
            "methodology-label",
            [
                "这场追逐，依据什么？",
                "What is this race based on?",
                "この比較の根拠は？",
                "¿En qué se basa esta comparación?",
            ],
        ),
        (
            "methodology-text",
            [
                "历史动画是原理示意，不是原物复原。日期以论文、博物馆和官方资料为依据；早期历史压缩播放，神经网络与重要转折放慢展示。模型登场时间和评测成绩公开时间分别记录，不会提前使用未来的成绩。",
                "Historical scenes are explanatory illustrations, not reconstructions. Dates follow papers, museums and official records. Early history is compressed; neural networks and major transitions receive more time. Releases and score publication dates are separate: future scores never enter past rankings.",
                "歴史画面は原理のイラストで、実物の復元ではありません。論文・博物館・公式資料を参照し、重要な転換点はゆっくり表示します。公開日と成績発表日を分け、未来の成績を過去の順位に使いません。",
                "Las escenas son ilustraciones explicativas, no reconstrucciones. Usamos artículos, museos y fuentes oficiales. Los hitos clave avanzan más despacio. Las fechas de lanzamiento y publicación de resultados son distintas: no usamos resultados futuros en clasificaciones pasadas.",
            ],
        ),
        (
            "methodology-score",
            [
                "只在相同指标和所标注的评测设置内排序。结果来自公开报告，并非本站独立复测；同组结果也可能存在实现差异。换赛道会重新排序，缺少可比成绩的模型不计名次。星体大小不代表参数量、意识或通用智能。",
                "Ranking stays within the named benchmark and protocol. Results are published reports, not independent tests by this site; implementation differences may remain. Changing tracks resets the comparison. Models without comparable scores remain unranked. Star size represents neither parameters, consciousness nor general intelligence.",
                "同じ指標と記載された評価条件の中だけで比較します。公開報告に基づき、本サイトの独立評価ではありません。実装差が残る場合もあります。比較可能な成績がなければ順位を付けず、星の大きさは知能や意識を表しません。",
                "Solo comparamos el indicador y protocolo indicados. Son resultados publicados, no pruebas independientes del sitio; pueden persistir diferencias de implementación. Los modelos sin resultados comparables no se clasifican. El tamaño de las estrellas no representa parámetros, conciencia ni inteligencia general.",
            ],
        ),
        (
            "methodology-source",
            [
                "查看数据与来源 ↗",
                "Read the data and sources ↗",
                "データと出典を見る ↗",
                "Consultar datos y fuentes ↗",
            ],
        ),
        (
            "footer-text",
            [
                "为好奇心而造。画面与原创音乐在浏览器中实时生成。",
                "Made for curiosity. Original visuals and music, generated live in your browser.",
                "好奇心のために。映像とオリジナル音楽をブラウザー内でリアルタイム生成。",
                "Hecho para la curiosidad. Imágenes y música original generadas en tu navegador.",
            ],
        ),
    ];
    for (id, values) in strings {
        text(id, tr(l, values));
    }
    let methodology = element("methodology-score")
        .text_content()
        .unwrap_or_default();
    text(
        "methodology-score",
        &format!("{methodology}\n{}", coverage_note(l)),
    );
    if let Some(root) = document().document_element() {
        let _ = root.set_attribute("lang", LANGUAGES[l]);
        let _ = root.set_attribute("class", if state.large { "large-text" } else { "" });
    }
    document().set_title(tr(
        l,
        [
            "AiHistory · 从工具到智能",
            "AiHistory · From tools to intelligence",
            "AiHistory · 道具から知能へ",
            "AiHistory · De herramientas a inteligencia",
        ],
    ));
    element("language")
        .unchecked_into::<HtmlSelectElement>()
        .set_value(LANGUAGES[l]);
    attr(
        "search",
        "placeholder",
        tr(
            l,
            [
                "搜索模型、机构或年份",
                "Model, organization or year",
                "モデル・組織・年を検索",
                "Modelo, organización o año",
            ],
        ),
    );
    attr(
        "search",
        "aria-label",
        tr(
            l,
            [
                "搜索模型",
                "Search models",
                "モデルを検索",
                "Buscar modelos",
            ],
        ),
    );
    attr(
        "text-size",
        "aria-label",
        tr(
            l,
            [
                "切换大字模式",
                "Toggle larger text",
                "文字サイズを切り替え",
                "Cambiar tamaño de texto",
            ],
        ),
    );
    attr(
        "restart",
        "aria-label",
        tr(l, ["从头开始", "Restart", "最初から", "Reiniciar"]),
    );
    attr(
        "fullscreen",
        "aria-label",
        tr(l, ["全屏观看", "Fullscreen", "全画面", "Pantalla completa"]),
    );
    attr(
        "motion-toggle",
        "aria-label",
        tr(
            l,
            [
                "减弱动态效果",
                "Reduce motion",
                "動きを減らす",
                "Reducir movimiento",
            ],
        ),
    );
    attr("progress", "max", &total_duration().to_string());
    text(
        "journey-meta",
        &format!(
            "{}  /  {}  /  {}",
            tr(
                l,
                [
                    "16 个历史瞬间",
                    "16 historical moments",
                    "16の歴史的瞬間",
                    "16 momentos históricos"
                ]
            ),
            tr(l, ["4 种语言", "4 languages", "4言語", "4 idiomas"]),
            clock_label(total_duration())
        ),
    );
    text(
        "transport-note",
        tr(
            l,
            [
                "连续观看，也可以随时暂停。画面为原理示意。",
                "Watch continuously, or pause to explore. Scenes illustrate ideas.",
                "続けて見る、止めて探る。画面は原理のイラストです。",
                "Sigue el recorrido o pausa para explorar. Las escenas ilustran ideas.",
            ],
        ),
    );
    render_timeline(state);
    render_atlas(state);
    render_story(state);
    render_controls(state);
}

fn render_controls(state: &State) {
    let l = state.lang;
    text(
        "play-toggle",
        if state.playing {
            tr(l, ["Ⅱ 暂停", "Ⅱ Pause", "Ⅱ 停止", "Ⅱ Pausa"])
        } else {
            tr(l, ["▶ 继续", "▶ Play", "▶ 再生", "▶ Seguir"])
        },
    );
    attr(
        "play-toggle",
        "aria-label",
        if state.playing {
            tr(l, ["暂停", "Pause", "一時停止", "Pausar"])
        } else {
            tr(l, ["播放", "Play", "再生", "Reproducir"])
        },
    );
    pressed("sound-toggle", state.sound);
    pressed("narration-toggle", state.narration);
    pressed("mode-easy", !state.deep);
    pressed("mode-deep", state.deep);
    pressed("text-size", state.large);
    pressed("motion-toggle", state.reduced);
    attr(
        "sound-toggle",
        "aria-label",
        tr(
            l,
            [
                "切换声音",
                "Toggle sound",
                "音の切り替え",
                "Activar o silenciar sonido",
            ],
        ),
    );
    text("sound-toggle", if state.sound { "♫" } else { "♪ ×" });
    text(
        "narration-toggle",
        tr(l, ["朗读", "Narrate", "読み上げ", "Narrar"]),
    );
    attr(
        "narration-toggle",
        "title",
        tr(
            l,
            [
                "使用设备提供的语音，音色与语言支持因设备而异",
                "Uses your device's available voices",
                "端末で利用できる音声を使用",
                "Usa las voces disponibles en tu dispositivo",
            ],
        ),
    );
    hidden(
        "audio-gate",
        !state.playing || !state.sound || state.music.as_ref().is_none_or(Music::is_running),
    );
    let canvas = element("film");
    let _ = canvas.set_attribute("data-playing", if state.playing { "true" } else { "false" });
    let _ = canvas.set_attribute(
        "data-audio-state",
        if state.music.as_ref().is_some_and(Music::is_running) {
            "running"
        } else {
            "suspended"
        },
    );
    let _ = canvas.set_attribute("data-time", &format!("{:.3}", state.time));
    element("progress")
        .unchecked_into::<HtmlInputElement>()
        .set_value(&format!("{:.1}", state.time));
    text(
        "time-display",
        &format!(
            "{} / {}",
            clock_label(state.time),
            clock_label(total_duration())
        ),
    );
    attr(
        "progress",
        "aria-valuetext",
        &format!(
            "{} / {}",
            clock_label(state.time),
            clock_label(total_duration())
        ),
    );
    attr(
        "progress",
        "style",
        &format!("--progress:{:.2}%", state.time / total_duration() * 100.0),
    );
}

fn render_story(state: &State) {
    let l = state.lang;
    let historical = history_at(state.time);
    text(
        "story-label",
        if state.time >= history_duration() + RACE_SECONDS {
            tr(
                l,
                [
                    "最新收录 · 查阅快照",
                    "LATEST OBSERVED SNAPSHOT",
                    "最新の観測スナップショット",
                    "ÚLTIMA INSTANTÁNEA OBSERVADA",
                ],
            )
        } else {
            tr(
                l,
                [
                    "此刻，能力发生改变",
                    "A NEW POSSIBILITY",
                    "新しい可能性",
                    "UNA NUEVA POSIBILIDAD",
                ],
            )
        },
    );
    hidden("capability-box", historical.is_none());
    hidden("ranking-box", historical.is_some());
    hidden("jump-race", historical.is_none());
    hidden("story-detail", !state.deep);
    hidden("race-hud", historical.is_some());
    attr(
        "stage-shell",
        "data-visual",
        &historical.map_or_else(|| "cosmos".to_owned(), |i| HISTORY[i].visual.to_string()),
    );
    attr(
        "stage-shell",
        "class",
        if historical.is_some_and(|i| HISTORY[i].visual != 9) {
            "stage-shell"
        } else {
            "stage-shell space-stage"
        },
    );
    if let Some(index) = historical {
        let h = &HISTORY[index];
        text("stage-date", h.date_label[l]);
        text("scene-kicker", h.date_label[l]);
        text("scene-caption", h.title[l]);
        if l == 0
            && let Some((head, tail)) = h.title[l].split_once('，')
        {
            element("scene-caption").set_inner_html(&format!(
                "<span class=title-phrase>{}，</span><span class=title-phrase>{}</span>",
                escape_html(head),
                escape_html(tail)
            ));
        }
        text("scene-ability", h.capability[l]);
        text("scene-index", &format!("{:02} — 16", index + 1));
        text(
            "story-number",
            &format!("{:02} / {:02}", index + 1, HISTORY.len()),
        );
        text("story-title", h.title[l]);
        text("story-summary", h.simple[l]);
        text("story-detail", h.detail[l]);
        text("capability-text", h.capability[l]);
        text("limit-text", h.limitation[l]);
        attr("story-source", "href", h.source_url);
        text(
            "story-source",
            &format!(
                "{} · {} ↗",
                tr(l, ["历史来源", "Source", "出典", "Fuente"]),
                h.source_name
            ),
        );
        attr(
            "film",
            "aria-label",
            &format!("{}: {}", h.title[l], h.simple[l]),
        );
    } else {
        let date = race_date(state.time);
        let year = date.floor() as u32;
        let month = ((date.fract() * 12.0).floor() as u32 + 1).min(12);
        text("stage-date", &format!("{year} · {month:02}"));
        text(
            "scene-kicker",
            tr(
                l,
                [
                    "模型星河",
                    "THE MODEL GALAXY",
                    "モデルの星河",
                    "GALAXIA DE MODELOS",
                ],
            ),
        );
        text(
            "scene-caption",
            tr(
                l,
                [
                    "每一次登场，都拓展可能。",
                    "Every arrival opens a possibility.",
                    "新しい登場、新しい可能性。",
                    "Cada llegada abre posibilidades.",
                ],
            ),
        );
        let available = MODELS
            .iter()
            .filter(|m| date_value(m.released) <= date)
            .count();
        text("story-number", &format!("{available} / {}", MODELS.len()));
        if let Some(i) = current_model(state) {
            let m = &MODELS[i];
            text("scene-caption", m.name);
            text(
                "scene-kicker",
                &format!(
                    "{} / {}",
                    if state.time >= history_duration() + RACE_SECONDS
                        && state.selected_model.is_none()
                    {
                        tr(
                            l,
                            [
                                "当前赛道领先",
                                "TRACK LEADER",
                                "この指標の首位",
                                "LÍDER DEL INDICADOR",
                            ],
                        )
                    } else {
                        tr(
                            l,
                            ["模型登场", "MODEL ARRIVAL", "モデル登場", "NUEVO MODELO"],
                        )
                    },
                    if state.time >= history_duration() + RACE_SECONDS
                        && state.selected_model.is_none()
                    {
                        SNAPSHOT_DATE
                    } else {
                        m.released
                    }
                ),
            );
            text("story-title", m.name);
            text("story-summary", m.summary[l]);
            let score_available = m.score.is_some() && date_value(m.reported) <= date;
            let result_detail = if score_available {
                format!(
                    "{}: {}\n{}: {}",
                    if m.score_observed {
                        tr(
                            l,
                            [
                                "查阅快照",
                                "Snapshot observed",
                                "参照スナップショット",
                                "Instantánea consultada",
                            ],
                        )
                    } else {
                        tr(
                            l,
                            [
                                "评测公布",
                                "Result published",
                                "成績発表",
                                "Resultado publicado",
                            ],
                        )
                    },
                    m.reported,
                    m.benchmark,
                    m.protocol
                )
            } else {
                tr(l, ["这个时点没有收录的可比评测成绩，不计名次。", "No comparable published result is included at this point; this model is unranked.", "この時点で収録された比較可能な成績はなく、順位を付けません。", "En este momento no hay resultados comparables incluidos; el modelo no se clasifica."]).to_owned()
            };
            text(
                "story-detail",
                &format!("{} · {}\n{result_detail}", m.organization, m.released),
            );
            attr(
                "story-source",
                "href",
                if state.deep && score_available {
                    m.score_source
                } else {
                    m.source_url
                },
            );
            text(
                "story-source",
                &format!(
                    "{} · {} ↗",
                    if state.deep && score_available {
                        tr(
                            l,
                            [
                                "评测来源",
                                "Evaluation source",
                                "評価の出典",
                                "Fuente de la evaluación",
                            ],
                        )
                    } else {
                        tr(
                            l,
                            [
                                "查看原始资料",
                                "Original source",
                                "一次資料を見る",
                                "Fuente original",
                            ],
                        )
                    },
                    if state.deep && score_available {
                        if m.score_observed {
                            "Mercor"
                        } else {
                            m.benchmark
                        }
                    } else {
                        m.organization
                    }
                ),
            );
        } else {
            text(
                "story-title",
                tr(
                    l,
                    [
                        "新的星河，正在形成",
                        "A new galaxy takes shape",
                        "新しい星河の始まり",
                        "Una nueva galaxia toma forma",
                    ],
                ),
            );
            text(
                "story-summary",
                tr(
                    l,
                    [
                        "时间继续前进，新的模型将陆续出现。",
                        "As time moves forward, new models will arrive.",
                        "時間が進むと、新しいモデルが登場します。",
                        "Con el paso del tiempo llegarán nuevos modelos.",
                    ],
                ),
            );
            text("story-detail", "");
            attr("story-source", "href", "https://arxiv.org/abs/1706.03762");
            text("story-source", "Attention Is All You Need ↗");
        }
        render_ranking(state);
        attr(
            "film",
            "aria-label",
            &format!(
                "{} {year}-{month:02}",
                tr(
                    l,
                    [
                        "模型发布时间与能力排名动画",
                        "Model release and capability ranking animation",
                        "モデルの登場と能力比較",
                        "Lanzamientos y comparación de modelos"
                    ]
                )
            ),
        );
    }
    text(
        "scene-summary",
        &element("story-summary").text_content().unwrap_or_default(),
    );
    if historical.is_none() {
        let track = track_index(state);
        text(
            "race-hud-track",
            match track {
                0 => "MMLU / 5-shot",
                1 => "MMLU / 0-shot CoT",
                _ => "AdvancedIF / 2026-10-24",
            },
        );
        text("scene-index", &format!("2017 — {}", race_end().floor()));
        text(
            "scene-ability",
            tr(
                l,
                [
                    "星轨向前，能力边界不断延伸",
                    "New arrivals, new possibilities",
                    "新しい登場が、可能性を広げる",
                    "Nuevas llegadas, nuevas posibilidades",
                ],
            ),
        );
    }
    for i in 0..HISTORY.len() {
        if let Some(stop) = document().get_element_by_id(&format!("event-{i}")) {
            let _ = stop.set_attribute(
                "aria-current",
                if historical == Some(i) {
                    "step"
                } else {
                    "false"
                },
            );
        }
    }
    if let Some(stop) = document().get_element_by_id("event-race") {
        let _ = stop.set_attribute(
            "aria-current",
            if historical.is_none() {
                "step"
            } else {
                "false"
            },
        );
    }
    let current = historical.map_or_else(|| "race".to_owned(), |i| i.to_string());
    let timeline = element("timeline-list");
    if timeline.get_attribute("data-current").as_deref() != Some(&current) {
        let _ = timeline.set_attribute("data-current", &current);
        let stop = element(&format!("event-{current}")).unchecked_into::<web_sys::HtmlElement>();
        let left = stop.offset_left()
            - timeline
                .unchecked_ref::<web_sys::HtmlElement>()
                .offset_left();
        timeline
            .set_scroll_left((left - timeline.client_width() / 2 + stop.client_width() / 2).max(0));
    }
}

fn render_ranking(state: &State) {
    let l = state.lang;
    let track = track_index(state);
    let (benchmark, protocol) = TRACKS[track];
    let snapshot_label = tr(
        l,
        [
            "查阅快照",
            "Observed snapshot",
            "参照スナップショット",
            "Instantánea consultada",
        ],
    );
    let active_label = if track == 2 {
        format!("{benchmark} · {protocol}\n{snapshot_label} · 2026-10-24")
    } else {
        format!("{benchmark} · {protocol}")
    };
    text("active-track", &active_label);
    let ranked = ranking(state.time, benchmark, protocol);
    let mut html = String::new();
    for (place, i) in ranked.iter().take(5).enumerate() {
        let m = &MODELS[*i];
        html.push_str(&format!(r#"<button class="rank-row" data-action="model" data-index="{i}"><span class="rank-place">{:02}</span><span class="model-dot" style="--model-color:{}"></span><span class="rank-name">{}</span><strong>{:.1}</strong></button>"#, place + 1, m.color, escape_html(m.name), m.score.unwrap_or(0.0)));
    }
    if html.is_empty() {
        html = format!(
            "<p class=rank-empty>{}</p>",
            tr(
                l,
                [
                    "这个时点还没有当前赛道的可比成绩。新模型仍会按发布时间登场。",
                    "No comparable results on this track yet. Models still arrive on their release dates.",
                    "この時点には比較可能な成績がありません。モデルは公開日に登場します。",
                    "Aún no hay resultados comparables. Los modelos aparecen en su fecha de lanzamiento."
                ]
            )
        );
    }
    element("leaderboard").set_inner_html(&html);
    let note = if track == 2 {
        tr(
            l,
            [
                "独立榜单查阅快照，不代表发布当天成绩；各模型使用榜单记录的最高推理设置，算力不同。",
                "Observed independent-board snapshot, not release-day results; highest-effort settings use different compute budgets.",
                "独立評価の参照スナップショットです。公開日当日の成績ではなく、最高推論設定の計算量はモデルごとに異なります。",
                "Instantánea independiente, no resultados del lanzamiento; los ajustes máximos usan distinto cómputo.",
            ],
        )
    } else {
        tr(
            l,
            [
                "公开报告成绩，非本站独立测评。星体间距表示当前赛道成绩，不代表通用智能。",
                "Published results, not our own tests. Positions reflect this benchmark, not general intelligence.",
                "公開報告の成績です。位置はこの指標を表し、汎用知能の順位ではありません。",
                "Resultados publicados, no pruebas propias. Las posiciones reflejan este indicador, no inteligencia general.",
            ],
        )
    };
    text("rank-note", &format!("{note}\n{}", coverage_note(l)));
}

fn render_timeline(state: &State) {
    let l = state.lang;
    let mut html = String::new();
    for (i, h) in HISTORY.iter().enumerate() {
        html.push_str(&format!(r#"<button class="timeline-stop" id="event-{i}" data-action="seek" data-index="{i}" aria-current="false"><span class="timeline-dot"></span><strong>{}</strong><span>{}</span></button>"#, escape_html(h.date_label[l]), escape_html(h.title[l])));
    }
    html.push_str(&format!(r#"<button class="timeline-stop race-stop" id="event-race" data-action="race" aria-current="false"><span class="timeline-dot">✦</span><strong>2017 → {:.0}</strong><span>{}</span></button>"#, race_end().floor(), tr(l, ["模型的星河", "The model galaxy", "モデルの星河", "Galaxia de modelos"])));
    element("timeline-list").set_inner_html(&html);
}

fn render_atlas(state: &State) {
    let l = state.lang;
    let query = state.query.to_lowercase();
    let mut html = String::new();
    let mut count = 0;
    for (i, model) in MODELS.iter().enumerate().filter(|(_, m)| {
        format!(
            "{} {} {} {}",
            m.name, m.organization, m.released, m.summary[l]
        )
        .to_lowercase()
        .contains(&query)
    }) {
        count += 1;
        html.push_str(&format!(r#"<button class="model-card" data-action="visit-model" data-index="{i}" style="--model-color:{}"><span class="model-card-top"><span class="model-orbit"><span></span></span><span class="model-year">{}</span></span><span class="model-company">{}</span><strong>{}</strong><span class="model-summary">{}</span><span class="model-card-bottom">{}<span aria-hidden="true">↗</span></span></button>"#, model.color, &model.released[..4], escape_html(model.organization), escape_html(model.name), escape_html(model.summary[l]), tr(l, ["回到登场时刻", "Visit its arrival", "登場した時点へ", "Viajar a su llegada"])));
    }
    if count == 0 {
        html = format!(
            "<p class=empty-results>{}</p>",
            tr(
                l,
                [
                    "没有找到这个模型，试试机构名称或年份。",
                    "No matches. Try an organization or year.",
                    "一致するモデルがありません。組織名や年をお試しください。",
                    "Sin resultados. Prueba una organización o un año."
                ]
            )
        );
    }
    element("model-grid").set_inner_html(&html);
    text("result-count", &count.to_string());
}

fn cancel_speech() {
    if let Ok(s) = window().speech_synthesis() {
        s.cancel();
    }
}
fn narrate(state: &mut State) {
    if !state.narration || !state.sound || !state.playing {
        return;
    }
    let (key, summary) = if let Some(index) = history_at(state.time) {
        (
            format!("history-{index}-{}", state.lang),
            HISTORY[index].simple[state.lang],
        )
    } else if let Some(index) = current_model(state) {
        (
            format!("model-{index}-{}", state.lang),
            MODELS[index].summary[state.lang],
        )
    } else {
        return;
    };
    if state.speak_key == key {
        return;
    }
    let Ok(synth) = window().speech_synthesis() else {
        return;
    };
    if synth.speaking() {
        synth.cancel();
    }
    let language = ["zh-CN", "en-US", "ja-JP", "es-ES"][state.lang];
    let voice = synth
        .get_voices()
        .iter()
        .filter_map(|v| v.dyn_into::<web_sys::SpeechSynthesisVoice>().ok())
        .find(|v| v.lang().to_lowercase().starts_with(LANGUAGES[state.lang]));
    if voice.is_none() {
        text(
            "transport-note",
            tr(
                state.lang,
                [
                    "设备暂无此语言的朗读声音；字幕与音乐仍可使用。",
                    "No device voice is available for this language. Captions and music remain available.",
                    "この言語の音声が端末にありません。字幕と音楽は利用できます。",
                    "Tu dispositivo no ofrece una voz para este idioma. Los textos y la música siguen disponibles.",
                ],
            ),
        );
        // Browser voices can arrive asynchronously; retry while this scene remains active.
        return;
    }
    if let Ok(utterance) = web_sys::SpeechSynthesisUtterance::new_with_text(summary) {
        utterance.set_lang(language);
        utterance.set_voice(voice.as_ref());
        utterance.set_rate(0.95);
        utterance.set_volume(0.8);
        synth.speak(&utterance);
        state.speech = Some(utterance);
        state.speak_key = key;
    }
}

fn resume_audio(state: &Shared) {
    if !state.borrow().playing || !state.borrow().sound {
        return;
    }
    let promise = state.borrow().music.as_ref().and_then(|m| m.resume().ok());
    if let Some(promise) = promise {
        let state = state.clone();
        spawn_local(async move {
            if JsFuture::from(promise).await.is_ok() {
                let mut s = state.borrow_mut();
                let time = s.music_time;
                let active = s.playing && s.sound;
                if let Some(music) = &mut s.music {
                    music.seek(time);
                    if !active {
                        music.pause();
                    }
                }
            }
            render_controls(&state.borrow());
        });
    }
}
fn pause(state: &mut State) {
    state.playing = false;
    if let Some(m) = &state.music {
        m.pause();
    }
    state.speak_key.clear();
    state.speech = None;
    cancel_speech();
}
fn seek(state: &mut State, time: f64) {
    state.time = time.clamp(0.0, total_duration());
    state.music_time = state.time % 96.0;
    if let Some(m) = &mut state.music {
        m.seek(state.music_time);
    }
    state.signature.clear();
    state.draw_signature.clear();
    state.needs_snap = true;
    state.speak_key.clear();
    state.selected_model = None;
    state.positions.fill(0.0);
    state.appearances.fill(0.0);
    state.speech = None;
    cancel_speech();
}
fn listen<F: 'static + FnMut(Event)>(
    target: &web_sys::EventTarget,
    name: &str,
    callback: F,
) -> Result<(), JsValue> {
    let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
    target.add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())?;
    callback.forget();
    Ok(())
}

fn bind_events(shared: &Shared) -> Result<(), JsValue> {
    let state = shared.clone();
    listen(document().as_ref(), "click", move |event| {
        let Some(target) = event
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .and_then(|e| e.closest("[data-action]").ok().flatten())
        else {
            return;
        };
        let action = target.get_attribute("data-action").unwrap_or_default();
        let index = target
            .get_attribute("data-index")
            .and_then(|i| i.parse::<usize>().ok());
        let mut resume = false;
        let mut full_render = false;
        {
            let mut s = state.borrow_mut();
            match action.as_str() {
                "inspect" => {
                    pause(&mut s);
                    s.deep = true;
                    attr("insight-panel", "open", "");
                    element("insight-panel").scroll_into_view();
                }
                "archive" => {
                    pause(&mut s);
                    attr("archive-panel", "open", "");
                }
                "play" => {
                    if s.playing {
                        pause(&mut s);
                    } else {
                        if s.time >= total_duration() {
                            seek(&mut s, 0.0);
                        }
                        s.selected_model = None;
                        s.signature.clear();
                        s.playing = true;
                        resume = true;
                    }
                }
                "unlock" => {
                    s.sound = true;
                    s.speak_key.clear();
                    resume = s.playing;
                }
                "restart" => {
                    seek(&mut s, 0.0);
                    s.playing = true;
                    resume = true;
                }
                "sound" => {
                    s.sound = !s.sound;
                    s.speak_key.clear();
                    resume = s.sound && s.playing;
                    if !s.sound {
                        s.speech = None;
                        if let Some(music) = &s.music {
                            music.pause();
                        }
                        cancel_speech();
                    }
                }
                "narration" => {
                    s.narration = !s.narration;
                    s.speak_key.clear();
                    if !s.narration {
                        s.speech = None;
                        cancel_speech();
                    }
                    resume = s.narration && s.playing && s.sound;
                }
                "motion" => {
                    s.reduced = !s.reduced;
                }
                "large" => {
                    s.large = !s.large;
                    save("aihistory.large", if s.large { "true" } else { "false" });
                    full_render = true;
                }
                "easy" => {
                    s.deep = false;
                }
                "deep" => {
                    s.deep = true;
                    pause(&mut s);
                }
                "race" => {
                    seek(&mut s, history_duration());
                    s.playing = true;
                    resume = true;
                }
                "seek" => {
                    if let Some(i) = index.filter(|i| *i < HISTORY.len()) {
                        seek(&mut s, history_start(i));
                        s.playing = true;
                        resume = true;
                    }
                }
                "model" => {
                    if let Some(i) = index.filter(|i| *i < MODELS.len()) {
                        s.selected_model = Some(i);
                        s.deep = true;
                        pause(&mut s);
                        attr("insight-panel", "open", "");
                    }
                }
                "visit-model" => {
                    if let Some(i) = index.filter(|i| *i < MODELS.len()) {
                        seek(&mut s, model_time(MODELS[i].released) + 1e-6);
                        s.selected_model = Some(i);
                        s.deep = true;
                        pause(&mut s);
                        attr("insight-panel", "open", "");
                        element("experience").scroll_into_view();
                    }
                }
                "fullscreen" => {
                    if document().fullscreen_element().is_some() {
                        document().exit_fullscreen();
                    } else {
                        let _ = element("stage-shell").request_fullscreen();
                    }
                }
                _ => {}
            }
            if let Some(music) = &s.music {
                music.set_volume(if s.sound { 0.7 } else { 0.0 });
            }
            if full_render {
                localize(&s);
            } else {
                render_controls(&s);
                render_story(&s);
            }
        }
        if resume {
            resume_audio(&state);
        }
    })?;
    let state = shared.clone();
    listen(document().as_ref(), "change", move |event| {
        let Some(target) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        else {
            return;
        };
        let mut s = state.borrow_mut();
        match target.id().as_str() {
            "language" => {
                s.lang = language_index(&target.value());
                save("aihistory.lang", LANGUAGES[s.lang]);
                s.speak_key.clear();
                s.speech = None;
                cancel_speech();
                localize(&s);
            }
            "speed" => {
                s.speed = target.value().parse::<f64>().unwrap_or(1.0).clamp(0.5, 2.0);
            }
            "benchmark" => {
                s.track = target
                    .value()
                    .parse::<usize>()
                    .ok()
                    .filter(|i| *i < TRACKS.len());
                s.signature.clear();
                s.draw_signature.clear();
                s.needs_snap = true;
                s.positions.fill(0.0);
                render_ranking(&s);
            }
            _ => {}
        }
    })?;
    let state = shared.clone();
    listen(document().as_ref(), "input", move |event| {
        let Some(target) = event
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let mut s = state.borrow_mut();
        match target.id().as_str() {
            "search" => {
                s.query = target.value();
                render_atlas(&s);
            }
            "progress" => {
                if let Ok(time) = target.value().parse::<f64>() {
                    pause(&mut s);
                    seek(&mut s, time);
                    render_controls(&s);
                    render_story(&s);
                }
            }
            _ => {}
        }
    })?;
    let state = shared.clone();
    listen(document().as_ref(), "keydown", move |event| {
        let Ok(event) = event.dyn_into::<KeyboardEvent>() else {
            return;
        };
        if event
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .is_some_and(|e| {
                matches!(
                    e.tag_name().as_str(),
                    "INPUT" | "SELECT" | "TEXTAREA" | "BUTTON" | "A"
                )
            })
        {
            return;
        }
        let mut resume = false;
        {
            let mut s = state.borrow_mut();
            match event.key().as_str() {
                " " => {
                    event.prevent_default();
                    if s.playing {
                        pause(&mut s);
                    } else {
                        if s.time >= total_duration() {
                            seek(&mut s, 0.0);
                        }
                        s.selected_model = None;
                        s.signature.clear();
                        s.playing = true;
                        resume = true;
                    }
                }
                "ArrowLeft" => {
                    event.prevent_default();
                    let time = s.time - 10.0;
                    seek(&mut s, time);
                }
                "ArrowRight" => {
                    event.prevent_default();
                    let time = s.time + 10.0;
                    seek(&mut s, time);
                }
                _ => return,
            }
            render_controls(&s);
            render_story(&s);
        }
        if resume {
            resume_audio(&state);
        }
    })?;
    let state = shared.clone();
    listen(document().as_ref(), "visibilitychange", move |_| {
        if document().hidden() {
            let mut s = state.borrow_mut();
            pause(&mut s);
            render_controls(&s);
        }
    })?;
    for id in ["insight-panel", "archive-panel"] {
        let state = shared.clone();
        listen(element(id).as_ref(), "toggle", move |_| {
            if element(id).has_attribute("open") {
                let mut s = state.borrow_mut();
                pause(&mut s);
                render_controls(&s);
            }
        })?;
    }
    Ok(())
}

fn start_frames(state: Shared, context: CanvasRenderingContext2d) -> Result<(), JsValue> {
    let canvas = context
        .canvas()
        .ok_or_else(|| JsValue::from_str("Canvas unavailable"))?;
    let callback: FrameCallback = Rc::new(RefCell::new(None));
    let next = callback.clone();
    *callback.borrow_mut() = Some(Closure::wrap(Box::new(move |now: f64| {
        {
            let mut s = state.borrow_mut();
            let delta = if s.last_frame == 0.0 {
                0.0
            } else {
                ((now - s.last_frame) / 1000.0).clamp(0.0, 0.1)
            };
            s.last_frame = now;
            let speaking = s.speech.is_some()
                && window()
                    .speech_synthesis()
                    .is_ok_and(|speech| speech.speaking() || speech.pending());
            if s.playing {
                let mut next_time = (s.time + delta * s.speed).min(total_duration());
                // Let the current explanation finish before replacing its caption.
                if s.narration && s.sound && speaking {
                    let boundary = if let Some(index) = history_at(s.time) {
                        history_start(index) + HISTORY[index].duration
                    } else {
                        let date = race_date(s.time);
                        MODELS
                            .iter()
                            .filter(|m| date_value(m.released) > date)
                            .map(|m| model_time(m.released))
                            .fold(total_duration(), f64::min)
                    };
                    next_time = next_time.min((boundary - 1e-6).max(s.time));
                }
                s.time = next_time;
                s.music_time = (s.music_time + delta) % 96.0;
                if s.time >= total_duration() {
                    pause(&mut s);
                }
            }
            let music_time = s.music_time;
            let playing = s.playing;
            let sound = s.sound;
            if let Some(music) = &mut s.music {
                music.set_volume(if !sound {
                    0.0
                } else if speaking {
                    0.22
                } else {
                    0.7
                });
                if playing && sound && music.is_running() {
                    music.tick(music_time);
                }
            }
            let early = history_at(s.time);
            let track = track_index(&s);
            if track != s.active_track {
                s.active_track = track;
                s.needs_snap = true;
            }
            let (metric, protocol) = TRACKS[track];
            let ranked = ranking(s.time, metric, protocol);
            let date = race_date(s.time);
            let mut visible: Vec<usize> = ranked.iter().copied().take(5).collect();
            let newest = MODELS
                .iter()
                .enumerate()
                .filter(|(_, m)| date_value(m.released) <= date)
                .map(|(i, _)| i)
                .next_back();
            if let Some(i) = s.selected_model.or(newest)
                && !visible.contains(&i)
            {
                visible.push(i);
            }
            if visible.is_empty() {
                visible.extend(
                    MODELS
                        .iter()
                        .enumerate()
                        .filter(|(_, m)| date_value(m.released) <= date)
                        .map(|(i, _)| i)
                        .take(6),
                );
            }
            for i in &visible {
                let valid = ranked.contains(i);
                let target = if valid {
                    MODELS[*i].score.unwrap_or(0.0) / 100.0
                } else {
                    0.18
                };
                let amount = if s.needs_snap || (s.playing && s.reduced) {
                    1.0
                } else if s.playing {
                    (delta * 2.8).min(1.0)
                } else {
                    0.0
                };
                s.positions[*i] += (target - s.positions[*i]) * amount;
                if s.needs_snap || (s.playing && s.reduced) {
                    s.appearances[*i] = 1.0;
                } else if s.playing {
                    s.appearances[*i] = s.appearances[*i]
                        .max(((s.time - model_time(MODELS[*i].released)) / 1.4).clamp(0.0, 1.0));
                }
            }
            s.needs_snap = false;
            let organizations: Vec<String> = visible
                .iter()
                .map(|i| {
                    if ranked.contains(i) {
                        MODELS[*i].organization.to_owned()
                    } else {
                        format!(
                            "{} · {}",
                            MODELS[*i].organization,
                            tr(
                                s.lang,
                                [
                                    "本赛道未排名",
                                    "Unranked here",
                                    "この比較対象外",
                                    "Sin clasificar aquí"
                                ]
                            )
                        )
                    }
                })
                .collect();
            let racers: Vec<scene::Racer<'_>> = visible
                .iter()
                .enumerate()
                .map(|(row, i)| {
                    let model = &MODELS[*i];
                    let rank = ranked.iter().position(|j| j == i).map_or(0, |r| r + 1);
                    scene::Racer {
                        name: model.name,
                        organization: &organizations[row],
                        color: model.color,
                        score: if rank > 0 { model.score } else { None },
                        position: s.positions[*i],
                        rank,
                        appeared: s.appearances[*i],
                    }
                })
                .collect();
            let visual_time = if s.reduced {
                early.map_or(0.0, |i| i as f64 * 3.0)
            } else {
                s.time
            };
            let compact = early.is_none()
                && window()
                    .inner_width()
                    .ok()
                    .and_then(|w| w.as_f64())
                    .is_some_and(|w| w <= 760.0);
            let (width, height) = if compact { (600, 500) } else { (1200, 675) };
            if canvas.width() != width || canvas.height() != height {
                canvas.set_width(width);
                canvas.set_height(height);
            }
            let draw_signature = format!(
                "{early:?}-{compact}-{:x}-{track}-{visible:?}-{ranked:?}-{}-{:?}",
                visual_time.to_bits(),
                s.lang,
                racers
                    .iter()
                    .map(|r| (r.position.to_bits(), r.appeared.to_bits()))
                    .collect::<Vec<_>>()
            );
            if draw_signature != s.draw_signature {
                if let Some(i) = early {
                    let local = if s.reduced {
                        1.0
                    } else {
                        (s.time - history_start(i)) / HISTORY[i].duration
                    };
                    let _ = scene::draw_history(&context, visual_time, HISTORY[i].visual, local);
                } else if compact {
                    let _ = crate::cosmos::draw_compact(&context, visual_time, &racers);
                } else {
                    let _ = crate::cosmos::draw(&context, visual_time, &racers);
                }
                s.draw_signature = draw_signature;
            }
            let month = date.floor() as u32 * 12 + (date.fract() * 12.0).floor() as u32;
            let inspected = current_model(&s);
            let published = inspected.is_some_and(|i| {
                MODELS[i].score.is_some() && date_value(MODELS[i].reported) <= date
            });
            let signature = format!(
                "{early:?}-{track}-{ranked:?}-{newest:?}-{month}-{inspected:?}-{published}"
            );
            if signature != s.signature {
                render_story(&s);
                s.signature = signature;
            }
            if now - s.last_dom > 200.0 {
                render_controls(&s);
                s.last_dom = now;
                if s.music.as_ref().is_some_and(Music::is_running) {
                    narrate(&mut s);
                }
            }
        }
        if let Some(callback) = next.borrow().as_ref() {
            let _ = window().request_animation_frame(callback.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>));
    window().request_animation_frame(
        callback
            .borrow()
            .as_ref()
            .expect("frame callback")
            .as_ref()
            .unchecked_ref(),
    )?;
    Ok(())
}
