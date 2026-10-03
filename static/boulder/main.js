// One Must Imagine — render/input shell. Owns the DOM; the sim is the wasm core.
// Talks to it only through tick()->snapshot and dispatch(command).

import init, { Game } from "./pkg/one_must_imagine.js";

const SAVE_KEY = "one-must-imagine/save";
const RENDER_MS = 120; // DOM refresh cadence (sim ticks every frame)
const FRAME_MS = 380; // ASCII heave animation cadence
const ROWS = 15;
const COLS = 64;
const SKY = 8;
const DAY_END = 5 / 6; // sun is up from Dawn through Dusk; Midnight is night
const PHASE_SECONDS = 120; // a 12-min day / 6 phases (mirrors sim::PHASE_SECONDS)
const STARS = [[0,6],[1,15],[0,25],[2,33],[1,46],[0,53],[2,60],[3,9],[1,38],[4,28],[2,19],[3,57],[5,48],[0,42],[4,12],[1,63],[5,35],[3,3]];

// per-phase sky top-colour (Dawn..Midnight), interpolated for a smooth cycle
const PHASE_SKY = [
  [0x3a, 0x22, 0x38], // dawn — muted rose
  [0x22, 0x34, 0x52], // morning — cool blue
  [0x36, 0x56, 0x7a], // noon — bright midday
  [0x2e, 0x3e, 0x5c], // afternoon — settling blue
  [0x5a, 0x30, 0x1c], // dusk — amber sunset
  [0x08, 0x0b, 0x16], // midnight — deep
];
// drifting, randomly-generated clouds
const CLOUD_SPRITES = [".-~-.", ".-~~-.", ".⌒⌒⌒.", "~-~-~", ".-~~~-."];
let clouds = [];
function spawnCloud(x) {
  return {
    x,
    y: 1 + Math.floor(Math.random() * Math.max(1, SKY - 4)),
    speed: 1.4 + Math.random() * 2.6,
    sprite: CLOUD_SPRITES[Math.floor(Math.random() * CLOUD_SPRITES.length)],
  };
}
function updateClouds(dt) {
  if (clouds.length === 0) for (let i = 0; i < 3; i++) clouds.push(spawnCloud(Math.random() * COLS));
  for (const c of clouds) {
    c.x += (c.speed * dt) / 1000;
    if (c.x > COLS + 8) Object.assign(c, spawnCloud(-8));
  }
}

const _lerp = (a, b, t) => Math.round(a + (b - a) * t);
function skyColor(f) {
  const p = ((f % 1) + 1) % 1 * 6;
  const i = Math.floor(p) % 6;
  const t = p - Math.floor(p);
  const a = PHASE_SKY[i], b = PHASE_SKY[(i + 1) % 6];
  return `rgb(${_lerp(a[0], b[0], t)},${_lerp(a[1], b[1], t)},${_lerp(a[2], b[2], t)})`;
}
// sun/night opacity crossfade — dusk is a real twilight, not a hard cut
function sunOp(f) {
  if (f < 1 / 6) return f * 6;                 // dawn: fade in as it rises
  if (f < 4 / 6) return 1;                      // morning..afternoon
  if (f < 5 / 6) return 1 - (f - 4 / 6) * 6;    // dusk: fade out (sunset)
  return 0;                                     // midnight
}
function nightOp(f) {
  if (f < 1 / 6) return 1 - f * 6;              // dawn: stars/moon fade out
  if (f < 4 / 6) return 0;                      // day
  if (f < 5 / 6) return (f - 4 / 6) * 6;        // dusk: they come in
  return 1;                                     // midnight
}
// a random moon phase, chosen once per night
const MOON_PHASES = ["●", "◑", "◐", "☽", "☾", "◗", "◖"];
let currentMoon = "☾";
let moonActive = false;

await init();
const game = new Game(localStorage.getItem(SAVE_KEY) || "", Date.now());

const $ = (id) => document.getElementById(id);
let activeTab = "climb";
let shopFilter = "all";     // upgrade-shop category filter
let shopSort = "category";  // upgrade-shop sort order
let scornSub = "general";   // Scorn shop subtab (general | stance)
let defianceSub = "condition"; // Defiance shop subtab
let frame = 0;

// Only touch the DOM when a section's markup actually changed — otherwise a
// hovered button gets destroyed and recreated every frame (the hover flash).
const _cache = {};
// Escape a string for safe use inside a double-quoted HTML attribute.
const escAttr = (t) => String(t).replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
function setHTML(id, html) {
  if (_cache[id] !== html) {
    $(id).innerHTML = html;
    _cache[id] = html;
  }
}

// ── floating effort numbers (the payoff for every push) ───────────────────────
const REDUCED = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
let _lastAction = null; // last seen manual-action count, to fire one float per hit
let _lastCascade = null; // last seen Cascade proc counter, to fire a burst float
let _lastRollback = null; // last seen roll-back count, to fire a scorn crit-float
// how close this effort was to the personal best decides how it looks — faded
// when weak, blazing near-best, and a full CRIT the instant you set a new record.
function floatTier(frac, best) {
  if (best) return "crit"; // any genuine new best gets the full spectacle
  if (frac >= 0.9) return "blaze";
  if (frac >= 0.65) return "strong";
  if (frac >= 0.35) return "mid";
  return "weak";
}
function spawnFloat(snap) {
  const layer = $("float-layer");
  if (!layer) return;
  const tier = floatTier(snap.last_frac, snap.last_best);
  const isHeave = snap.last_kind === "heave";

  // anchor above the pusher; jitter so it never lands in the same spot twice
  const lr = layer.getBoundingClientRect();
  let cx = lr.width * 0.5, cy = lr.height * 0.42;
  const fig = $("scene").querySelector(".figure");
  if (fig) {
    const fr = fig.getBoundingClientRect();
    cx = fr.left - lr.left + fr.width / 2;
    cy = fr.top - lr.top;
  }
  cx += (Math.random() - 0.5) * 54;
  cy += -14 - Math.random() * 16;
  const dx = ((Math.random() - 0.5) * 30).toFixed(0);

  const el = document.createElement("div");
  el.className = `float ${tier}${isHeave ? " heave" : ""}`;
  el.style.left = `${cx}px`;
  el.style.top = `${cy}px`;
  el.style.setProperty("--dx", `${dx}px`);
  const glyph = isHeave ? "◆ " : "";
  el.innerHTML = snap.last_best
    ? `<span class="float-num">${glyph}+${snap.last_effort}</span><span class="float-tag">${isHeave ? "HEAVE · NEW BEST" : "NEW BEST"}</span>`
    : `<span class="float-num">${glyph}+${snap.last_effort}</span>`;
  layer.appendChild(el);
  const life = tier === "crit" ? (isHeave ? 2000 : 1700) : 1200;
  setTimeout(() => el.remove(), life);

  if (tier !== "crit" || REDUCED) return;
  // the payoff of payoffs: a shock ring, radiating sparks, and a scene kick.
  // Heave lands heavier than push — a second ring, more sparks, a bigger shake.
  const heavy = isHeave;
  const rings = heavy ? 2 : 1;
  for (let r = 0; r < rings; r++) {
    const ring = document.createElement("div");
    ring.className = `float-ring${heavy ? " heave" : ""}`;
    ring.style.left = `${cx}px`;
    ring.style.top = `${cy}px`;
    if (r === 1) ring.style.animationDelay = "0.12s";
    layer.appendChild(ring);
    setTimeout(() => ring.remove(), 1000);
  }
  const sparks = heavy ? 14 : 8;
  for (let i = 0; i < sparks; i++) {
    const ang = (Math.PI * 2 * i) / sparks + Math.random() * 0.5;
    const dist = (heavy ? 74 : 58) + Math.random() * 52;
    const sp = document.createElement("div");
    sp.className = `float-spark${heavy ? " heave" : ""}`;
    sp.style.left = `${cx}px`;
    sp.style.top = `${cy}px`;
    sp.style.setProperty("--sx", `${(Math.cos(ang) * dist).toFixed(0)}px`);
    sp.style.setProperty("--sy", `${(Math.sin(ang) * dist).toFixed(0)}px`);
    layer.appendChild(sp);
    setTimeout(() => sp.remove(), 820);
  }
  const scene = $("scene");
  const shake = heavy ? "crit-shake-heave" : "crit-shake";
  scene.classList.remove("crit-shake", "crit-shake-heave");
  void scene.offsetWidth; // reflow so the animation restarts on rapid crits
  scene.classList.add(shake);
  setTimeout(() => scene.classList.remove(shake), heavy ? 620 : 520);
}

// Cascade auto-burst float — the auto-flavoured cousin of the push/heave floats
function spawnCascadeFloat(snap) {
  const layer = $("float-layer");
  if (!layer || !snap.cascade_effort) return;
  const lr = layer.getBoundingClientRect();
  let cx = lr.width * 0.5, cy = lr.height * 0.42;
  const fig = $("scene").querySelector(".figure");
  if (fig) {
    const fr = fig.getBoundingClientRect();
    cx = fr.left - lr.left + fr.width / 2;
    cy = fr.top - lr.top;
  }
  cx += (Math.random() - 0.5) * 48;
  cy += -12 - Math.random() * 14;
  const el = document.createElement("div");
  el.className = "float cascade";
  el.style.left = `${cx}px`;
  el.style.top = `${cy}px`;
  el.style.setProperty("--dx", `${((Math.random() - 0.5) * 26).toFixed(0)}px`);
  el.innerHTML = `<span class="float-num">⚙ +${snap.cascade_effort}</span>`;
  layer.appendChild(el);
  setTimeout(() => el.remove(), 1100);
}

// Scorn crit-float — fires on a roll-back, anchored to the scorn counter itself,
// tiered by how the gain ranks vs the per-run best (faint → blazing → full CRIT
// when you beat it). Positioned fixed at the counter so it works on any tab.
function spawnScornFloat(snap) {
  const anchor = document.querySelector(".v-scorn");
  if (!anchor || !snap.last_scorn) return;
  const r = anchor.getBoundingClientRect();
  const tier = floatTier(snap.last_scorn_frac, snap.last_scorn_best);
  const cx = r.left + r.width / 2, cy = r.top;

  const el = document.createElement("div");
  el.className = `scorn-float ${tier}`;
  el.style.left = `${cx}px`;
  el.style.top = `${cy}px`;
  el.style.setProperty("--dx", `${((Math.random() - 0.5) * 20).toFixed(0)}px`);
  el.innerHTML = snap.last_scorn_best
    ? `<span class="scorn-float-num">+${snap.last_scorn} ◆</span><span class="scorn-float-tag">RECORD HAUL</span>`
    : `<span class="scorn-float-num">+${snap.last_scorn} ◆</span>`;
  document.body.appendChild(el);
  setTimeout(() => el.remove(), tier === "crit" ? 1900 : 1150);

  // the counter itself pulses; a new record adds a ring + spark burst around it
  anchor.classList.remove("scorn-pop");
  void anchor.offsetWidth;
  anchor.classList.add("scorn-pop");
  setTimeout(() => anchor.classList.remove("scorn-pop"), 640);
  if (tier !== "crit" || REDUCED) return;
  const ring = document.createElement("div");
  ring.className = "scorn-ring";
  ring.style.left = `${cx}px`;
  ring.style.top = `${r.top + r.height / 2}px`;
  document.body.appendChild(ring);
  setTimeout(() => ring.remove(), 900);
  for (let i = 0; i < 10; i++) {
    const ang = (Math.PI * 2 * i) / 10 + Math.random() * 0.5;
    const dist = 40 + Math.random() * 40;
    const sp = document.createElement("div");
    sp.className = "scorn-spark";
    sp.style.left = `${cx}px`;
    sp.style.top = `${r.top + r.height / 2}px`;
    sp.style.setProperty("--sx", `${(Math.cos(ang) * dist).toFixed(0)}px`);
    sp.style.setProperty("--sy", `${(Math.sin(ang) * dist).toFixed(0)}px`);
    document.body.appendChild(sp);
    setTimeout(() => sp.remove(), 780);
  }
}

// ── unlock flashes (the dopamine) ─────────────────────────────────────────────
let _prevAuto = null;
const _prevTab = {};
const TAB_SUB = { scorn: "spend it on upgrades", defiance: "permanent upgrades", universes: "costs defiance" };
function flash(title, sub) {
  const el = document.createElement("div");
  el.className = "flash";
  el.innerHTML = `<b>${title}</b>${sub ? `<span>${sub}</span>` : ""}`;
  $("console").appendChild(el);
  setTimeout(() => el.remove(), 2200);
}
function detectUnlocks(snap) {
  const a = snap.run.auto_unlocked;
  if (_prevAuto === null) {
    // first render: seed state, don't celebrate what was already unlocked
    _prevAuto = a;
    snap.tabs.forEach((t) => (_prevTab[t.key] = t.unlocked));
    return;
  }
  if (!_prevAuto && a) flash("⚙ AUTO-PUSH UNLOCKED", "it climbs on its own now");
  _prevAuto = a;
  for (const t of snap.tabs) {
    if (!_prevTab[t.key] && t.unlocked && t.key !== "climb") flash(`✦ ${t.name.toUpperCase()} UNLOCKED`, TAB_SUB[t.key] || "");
    _prevTab[t.key] = t.unlocked;
  }
}

// ── notice (offline / refused / corrupt / import) ─────────────────────────────
function notice(msg, bad) {
  let el = $("boulder-notice");
  if (!el) {
    el = document.createElement("div");
    el.id = "boulder-notice";
    el.className = "boulder-notice";
    document.querySelector(".console").before(el);
  }
  el.classList.toggle("bad", !!bad);
  el.textContent = msg;
  el.hidden = false;
}
(function showReport() {
  let r;
  try { r = JSON.parse(game.load_report()); } catch { return; }
  if (r.outcome === "resumed" && r.steps > 0) {
    const s = Math.round(r.offline_seconds);
    notice(`While you were away (${Math.floor(s / 3600)}h ${Math.floor((s % 3600) / 60)}m), the boulder kept rolling.`);
  } else if (r.outcome === "refused") {
    notice(`A save from a newer version (v${r.found}) is here and left untouched. Update, or Abandon to start fresh.`, true);
  } else if (r.outcome === "corrupt") {
    notice(`The stored save couldn't be read; it was left in place. Abandon to start fresh.`, true);
  }
})();

// ── the ASCII climb scene ─────────────────────────────────────────────────────
// A camera-scrolled world: the hill lengthens with every roll-back, and the view
// pans to keep the pusher in frame, so the summit only reveals itself as you near
// it. Three parallax planes give it depth — distant range, the climb, the sky.
const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
const smooth = (t) => t * t * (3 - 2 * t);
// how wide the world is (in cols) for a given roll-back count — grows, then caps
const worldColsFor = (rollbacks) => COLS + Math.min(Math.floor(rollbacks) * 6, 220);
// screen-row of the climb surface at world column wx — an organic ridgeline that
// sweeps from valley floor up to a peak, with foothill wobble that smooths to a
// clean summit
function ridgeRow(wx, worldCols) {
  const t = clamp(wx / (worldCols - 1), 0, 1);
  const base = (ROWS - 1) + (6 - (ROWS - 1)) * smooth(t);
  const wob = (Math.sin(wx * 0.45) + Math.sin(wx * 0.23 + 2.1) * 0.6) * (1 - t) * 0.9;
  // clamp to real rows — the wobble can overshoot past the grid at either end
  return clamp(Math.round(base - wob), 4, ROWS - 1);
}
// the distant range silhouette — jagged, near the horizon, drawn behind the climb
const rangeRow = (wx) => Math.round(4.4 - 1.7 * Math.abs(Math.sin(wx * 0.14)) - Math.sin(wx * 0.33 + 0.7) * 0.5);

function put(g, cls, y, x, s, cl) {
  for (let i = 0; i < s.length; i++) {
    const xx = x + i;
    if (s[i] !== " " && y >= 0 && y < ROWS && xx >= 0 && xx < COLS) {
      g[y][xx] = s[i];
      cls[y][xx] = cl || "";
    }
  }
}
function buildScene(progress, f, rollbacks) {
  const g = Array.from({ length: ROWS }, () => Array(COLS).fill(" "));
  const cls = Array.from({ length: ROWS }, () => Array(COLS).fill(""));
  const sOp = sunOp(f), nOp = nightOp(f);
  const worldCols = worldColsFor(rollbacks || 0);
  // camera follows the boulder; clamped so it never scrolls past the world edges
  const boulderWX = progress * (worldCols - 1);
  const cam = clamp(Math.round(boulderWX - COLS / 2), 0, Math.max(0, worldCols - COLS));

  // stars appear one by one as night deepens; the moon fades in (via --moon-op)
  if (nOp > 0.02) {
    // a faint Milky Way band, deepest at true midnight — drawn behind the bright
    // stars, only on empty sky (never over the moon or the crisp stars)
    if (nOp > 0.55) {
      const MILKY = [[0,10],[1,12],[0,14],[1,16],[2,17],[1,19],[0,21],[2,22],[1,24],[3,25],[2,27],[1,29],[3,30],[2,32],[4,33],[3,35],[2,37],[4,38],[3,40],[5,41],[4,43],[3,45],[5,46],[4,48]];
      const m = Math.round(((nOp - 0.55) / 0.45) * MILKY.length);
      for (let i = 0; i < m; i++) {
        const [y, x] = MILKY[i];
        if (g[y][x] === " ") put(g, cls, y, x, "·", "haze");
      }
    }
    const n = Math.round(nOp * STARS.length);
    for (let i = 0; i < n; i++) {
      const [y, x] = STARS[i];
      put(g, cls, y, x, (x + y) % 3 === 0 ? "*" : "·", `star s${i % 8}`);
    }
    put(g, cls, 2, COLS - 12, currentMoon, "moon");
  }
  // the sun — always high (never near the mountain/pusher), fades via --sun-op
  if (sOp > 0.02) {
    const arc = Math.sin(Math.PI * Math.min(1, f / (5 / 6)));
    const sx = Math.round(6 + (f / (5 / 6)) * (COLS - 17));
    const sy = Math.round(4 - 3 * arc);
    put(g, cls, sy - 1, sx - 2, "\\ | /", "sun");
    put(g, cls, sy, sx - 2, "─(◉)─", "sun");
    put(g, cls, sy + 1, sx - 2, "/ | \\", "sun");
  }

  // the main climb surface, per screen column — compute tops first so the distant
  // range and the clouds can be occluded by the ridge in front of them
  const tops = new Array(COLS);
  for (let sx = 0; sx < COLS; sx++) tops[sx] = ridgeRow(sx + cam, worldCols);

  // distant range — slow parallax (0.35×), only where the climb doesn't cover it
  for (let sx = 0; sx < COLS; sx++) {
    const rt = rangeRow(sx + Math.round(cam * 0.35));
    if (tops[sx] > rt + 1) {
      put(g, cls, rt, sx, "▔", "range");
      put(g, cls, rt + 1, sx, "▓", "range");
    }
  }
  // clouds drift across the daytime sky, occluded by peaks in front of them
  if (sOp > 0.05) {
    for (const c of clouds) {
      const cx = Math.round(c.x);
      for (let i = 0; i < c.sprite.length; i++) {
        const xx = cx + i;
        if (c.sprite[i] !== " " && xx >= 0 && xx < COLS && c.y < tops[xx]) {
          g[c.y][xx] = c.sprite[i]; cls[c.y][xx] = "cloud";
        }
      }
    }
  }
  // the climb itself — a depth-shaded mass, snow-capped where it breaks the sky
  for (let sx = 0; sx < COLS; sx++) {
    const top = tops[sx];
    const capped = top <= 8; // near the summit — dust it with snow
    g[top][sx] = "_"; cls[top][sx] = capped ? "snow" : "ridge";
    for (let y = top + 1; y < ROWS; y++) {
      const d = y - top;
      if (capped && d <= 2) { g[y][sx] = d < 2 ? "░" : "▒"; cls[y][sx] = "snow"; }
      else { g[y][sx] = d < 2 ? "░" : d < 4 ? "▒" : "▓"; cls[y][sx] = "mtn"; }
    }
  }
  // the summit flag — planted at the world's end, so it slides into view only as
  // the camera reaches the top of a long climb
  const flagSX = (worldCols - 1) - cam;
  if (flagSX >= 2 && flagSX < COLS) {
    const pk = tops[flagSX] ?? ridgeRow(worldCols - 1, worldCols);
    put(g, cls, pk - 1, flagSX - 2, "|__", "flag");
    put(g, cls, pk - 2, flagSX - 2, "|", "flag");
  }
  // the boulder, and the pusher leaning uphill into it
  const bsx = clamp(Math.round(boulderWX - cam), 2, COLS - 2);
  const bTop = ridgeRow(bsx + cam, worldCols);
  put(g, cls, bTop - 1, bsx, "●", "boulder");
  const pTop = ridgeRow(bsx - 2 + cam, worldCols);
  put(g, cls, pTop - 2, bsx - 1, "o", "figure");
  put(g, cls, pTop - 1, bsx - 2, "/|", "figure");
  put(g, cls, pTop, bsx - 2, "/ \\", "figure");
  // build HTML, wrapping coloured runs (our charset has no HTML specials)
  let out = "";
  for (let y = 0; y < ROWS; y++) {
    let line = "", run = "", rc = "";
    for (let x = 0; x < COLS; x++) {
      if (cls[y][x] !== rc) {
        if (run) line += rc ? `<span class="${rc}">${run}</span>` : run;
        run = ""; rc = cls[y][x];
      }
      run += g[y][x];
    }
    if (run) line += rc ? `<span class="${rc}">${run}</span>` : run;
    out += line + (y < ROWS - 1 ? "\n" : "");
  }
  return out;
}

// ── commands ──────────────────────────────────────────────────────────────────
function send(cmd) {
  game.dispatch(JSON.stringify(cmd));
  render();
}

// ── hold-to-push: press-and-hold ramps 2/s → 5/s over ~3s, release resets. No
// frantic clicking; holding IS the active playstyle (bound to click + Space).
// These fire real manual pushes (build momentum, keep Grind active) — the ramp
// is purely an input convenience, distinct from Rolling's passive auto-push.
let holdActive = false, holdStart = 0, holdTimer = null, pausedAt = 0;
let heldDown = false, region = null, ptrX = 0, ptrY = 0; // drag gesture state
let _pushRateMax = 5; // hold-to-push ceiling (Speed raises it); updated each render
function pushAvailable() {
  const btn = document.getElementById("b-push");
  return btn && !btn.disabled;
}
// Which action button is under the pointer? Live geometry, so it survives the
// buttons re-rendering mid-hold.
function regionAt(x, y) {
  const el = document.elementFromPoint(x, y);
  if (!el || !el.closest) return null;
  if (el.closest('button[data-act="push"]')) return "push";
  if (el.closest('button[data-act="heave"]')) return "heave";
  if (el.closest('button[data-act="rollback"]')) return "rollback";
  return null;
}
function holdFire() {
  if (!holdActive) return;
  send({ kind: "push" });
  // ramp 2/s → the Speed-raised cap over 3s of sustained holding
  const t = Math.min((performance.now() - holdStart) / 3000, 1);
  const rate = 2 + (_pushRateMax - 2) * t;
  holdTimer = setTimeout(holdFire, 1000 / rate);
}
function startHold() {
  // resumes firing from the CURRENT holdStart (fresh gestures set holdStart before
  // calling this; a heave detour leaves it alone so the ramp continues).
  if (holdActive || !pushAvailable()) return;
  holdActive = true;
  holdFire(); // fire one immediately, then ramp
}
function stopHold() {
  holdActive = false;
  if (holdTimer) { clearTimeout(holdTimer); holdTimer = null; }
}
// While the mouse is held, the button under the pointer drives the action: Push
// = ramping hold; Heave/Roll back = fire once on entry. Sliding OFF push freezes
// the ramp clock and sliding back resumes it where it was — no lost progress to a
// heave. Only fires on a region *change*, so resting on Heave doesn't spam it.
function setRegion(r) {
  if (r === region) return;
  const wasPush = region === "push";
  region = r;
  stopHold();
  if (wasPush) pausedAt = performance.now(); // start freezing the ramp clock
  if (r === "push") {
    if (pausedAt) { holdStart += performance.now() - pausedAt; pausedAt = 0; } // thaw: skip the paused span
    startHold();
  } else if (r === "heave") send({ kind: "heave" });
  else if (r === "rollback") send({ kind: "rollback" });
}
function endGesture() { heldDown = false; region = null; stopHold(); }

// ── the upgrade shop — categorized, filterable, sortable ──────────────────────
const CATS = {
  core:     { label: "Core",     color: "var(--u-core)",  order: 0 },
  manual:   { label: "Manual",   color: "var(--accent)",  order: 1 },
  auto:     { label: "Auto",     color: "var(--u-auto)",  order: 2 },
  momentum: { label: "Momentum", color: "var(--u-mom)",   order: 3 },
  heave:    { label: "Heave",    color: "var(--u-heave)", order: 4 },
  economy:  { label: "Economy",  color: "var(--u-econ)",  order: 5 },
};
// Which family each upgrade belongs to. Temporary home: it lives in the view
// while the roster is in flux (balancing). Fold into RunUpgradeDef.category and
// expose via the snapshot once the upgrade set settles.
const CAT_OF = {
  grip: "core", muscle: "core", leverage: "core",
  power: "manual", speed: "manual", streak: "manual",
  rolling: "auto", haste: "auto", cascade: "auto",
  reach: "momentum", conduit: "momentum",
  force: "heave", brawn: "heave", weight: "heave",
  spite: "economy", zeal: "economy", surplus: "economy", thrift: "economy",
};
const catOf = (key) => CAT_OF[key] || "core";
const INF_RANK = 100000; // max_level at/above this reads as infinite (∞)
// within-category order: infinite workhorse → scalable capped → one-time toggle
const srank = (u) => (u.max_level >= INF_RANK ? 0 : u.max_level <= 1 ? 2 : 1);
const SORTS = ["category", "cost", "afford"];
const SORT_LABEL = { category: "by category", cost: "by cost", afford: "affordable" };

function shopRow(u, act = "buy-upgrade") {
  const color = CATS[catOf(u.key)].color;
  const lv = u.maxed ? "maxed"
    : u.max_level >= INF_RANK ? `lv ${u.level}`
    : `lv ${u.level}/${u.max_level}`;
  const state = u.maxed ? "maxed" : u.locked ? "locked" : u.affordable ? "afford" : "";
  const label = u.maxed ? "MAX" : u.locked ? "locked" : `${u.cost} ◆`;
  const dis = u.maxed || u.locked || !u.affordable ? "disabled" : "";
  // current → next bonus, when the upgrade exposes one
  const bonus = u.effect
    ? `<span class="row-bonus">${u.effect}${u.maxed ? "" : ` <span class="row-arrow">→</span> <b>${u.effect_next}</b>`}</span>`
    : "";
  return `<li class="shop-row ${state}" style="--chip:${color}"><span class="row-spine"></span><span class="row-main"><span class="row-top"><b class="row-name">${u.name}</b><span class="row-lv">${lv}</span>${bonus}</span><span class="row-desc">${u.desc}</span></span><button class="row-buy" data-act="${act}" data-key="${u.key}" ${dis}>${label}</button></li>`;
}
// a defiance skill/phase node rendered as a card (name · rank · desc · buy)
function skillCard(n, act, dataAttrs) {
  const unlocked = n.unlocked !== false; // phase nodes are always unlocked
  const rank = n.maxed ? "MAX" : `rank ${n.rank}/${n.max_rank}`;
  const state = n.maxed ? "maxed" : !unlocked ? "locked" : n.affordable ? "afford" : "";
  const label = n.maxed ? "maxed" : !unlocked ? "◇ locked" : `${n.cost} ⟡`;
  const dis = n.maxed || !unlocked || !n.affordable ? "disabled" : "";
  return `<div class="skill-card ${state}"><div class="card-top"><b class="card-name">${n.name}</b><span class="card-rank">${rank}</span></div><p class="card-desc">${n.desc}</p><button class="card-buy" data-act="${act}" ${dataAttrs} ${dis}>${label}</button></div>`;
}

// a shared subtab bar (Defiance + Scorn shops). subs: [key, label, badge?][].
// Buttons carry `data-subtab="scope:key"`; the delegated handler routes on scope.
function subtabBar(scope, active, subs) {
  return `<div class="subtabs">${subs
    .map(([k, l, b]) => `<button class="subtab ${active === k ? "active" : ""}" data-subtab="${scope}:${k}">${l}${b != null && b !== "" ? ` <b>${b}</b>` : ""}</button>`)
    .join("")}</div>`;
}

// ── panels ────────────────────────────────────────────────────────────────────
const tab = (snap, key) => snap.tabs.find((t) => t.key === key) || { unlocked: false };

function climbPanel(snap) {
  const r = snap.run;
  let hint;
  if (!r.auto_unlocked) {
    const left = Math.max(0, 5 - r.pushes_this_run);
    hint = `Click and hold <b>${r.action_verb}</b> (or hold Space). ${left} more push${left === 1 ? "" : "es"} and the boulder rolls on its own — free.`;
  } else if (!tab(snap, "scorn").unlocked) {
    hint = `It rolls on its own now. Reach the <b>summit</b> — a 5s countdown starts, and the higher you climb past it, the more scorn you bank when it auto-rolls back.`;
  } else if (!tab(snap, "defiance").unlocked) {
    hint = `Spend scorn in the <b>Scorn</b> tab to climb faster. Roll back often — scorn compounds toward your first prestige.`;
  } else {
    hint = `Push to the summit, roll back for scorn, spend it to climb faster, prestige for defiance.`;
  }
  // personal bests — THIS run on the card (resets at prestige); hover shows the
  // all-time best with its stance and how many prestiges ago it was set.
  const bests = [];
  if (snap.run.best_push) bests.push(["best push", snap.run.best_push, "", snap.best_push_detail]);
  if (snap.run.best_heave) bests.push(["best heave", snap.run.best_heave, "◆", snap.best_heave_detail]);
  if (snap.run.best_scorn) bests.push(["best haul", snap.run.best_scorn, "◆", snap.best_scorn_detail]);
  const bestsHtml = bests.length
    ? `<div class="records"><div class="record-head">personal bests <span class="record-sub">· this run</span></div><div class="records-row">${bests
        .map(([l, v, g, d]) => `<span class="record${d ? " has-detail" : ""}"${d ? ` data-detail="${escAttr(d)}" tabindex="0" aria-label="${escAttr(`${g ? g + " " : ""}${l}: ${v}. ${d}`)}"` : ""}><span class="record-label">${g ? `${g} ` : ""}${l}</span><span class="record-val">${v}</span></span>`)
        .join("")}</div></div>`
    : "";

  const statCard = (s) => `<span class="stat${s.detail ? " has-detail" : ""}"${s.detail ? ` data-detail="${escAttr(s.detail)}" tabindex="0" aria-label="${escAttr(`${s.label}: ${s.value}. ${s.detail}`)}"` : ""}><span class="stat-label">${s.label}</span><span class="stat-val">${s.value}</span></span>`;

  // your stance — the innate identity + shop passive, in its own purple set
  const ss = snap.stance_stats || [];
  const stanceName = snap.run.stance ? snap.run.stance[0].toUpperCase() + snap.run.stance.slice(1) : "";
  const stanceHtml = ss.length
    ? `<div class="stat-block stance-block"><div class="stat-head">your stance — ${stanceName}</div><div class="stats">${ss.map(statCard).join("")}</div></div>`
    : "";

  const stats = snap.stats || [];
  const statsHtml = stats.length
    ? `<div class="stat-block"><div class="stat-head">your edge</div><div class="stats">${stats.map(statCard).join("")}</div></div>`
    : "";
  return `<p class="panel-hint">${hint}</p>${bestsHtml}${stanceHtml}${statsHtml}`;
}

function scornPanel(snap) {
  const r = snap.run;
  const ups = r.upgrades || [];
  const hasStance = r.stance_branch && r.stance_branch.length;
  if (scornSub === "stance" && !hasStance) scornSub = "general";
  const affordN = ups.filter((u) => u.affordable).length;

  // subtabs: General (the categorized upgrades) + Stance (only once committed)
  const subs = [["general", "General", ups.length]];
  if (hasStance) subs.push(["stance", `${r.stance} branch`, r.stance_branch.length]);
  let html = `<div class="shop-head"><div class="shop-wallet">scorn <b>${r.scorn}</b> ◆<span class="shop-afford">${affordN ? `${affordN} affordable` : "none affordable"}</span></div>${subtabBar("scorn", scornSub, subs)}</div>`;

  if (scornSub === "stance") {
    if (r.sub_build) {
      html += `<div class="subbuilds"><span class="ch-label">path locked in:</span> <b>${r.sub_build}</b></div>`;
    } else if (r.sub_build_choices && r.sub_build_choices.length) {
      html += `<div class="subbuilds"><span class="ch-label">choose a path</span>${r.sub_build_choices.map((c) => `<button class="stance-pick" data-act="choose-subbuild" data-key="${c.key}" title="${c.desc}">${c.name}</button>`).join("")}</div>`;
    }
    html += `<ul class="shop-items">${r.stance_branch.map((u) => shopRow(u, "buy-branch")).join("")}</ul>`;
    return html;
  }

  // General: category filter + sort + grouped upgrade rows
  const counts = {};
  for (const u of ups) counts[catOf(u.key)] = (counts[catOf(u.key)] || 0) + 1;
  const cats = Object.keys(CATS).filter((c) => counts[c]).sort((a, b) => CATS[a].order - CATS[b].order);
  if (shopFilter !== "all" && !cats.includes(shopFilter)) shopFilter = "all";
  const chip = (c) => c === "all"
    ? `<button class="shop-chip ${shopFilter === "all" ? "active" : ""}" data-filter="all">all <b>${ups.length}</b></button>`
    : `<button class="shop-chip ${shopFilter === c ? "active" : ""}" data-filter="${c}" style="--chip:${CATS[c].color}"><i class="chip-dot"></i>${CATS[c].label} <b>${counts[c]}</b></button>`;
  let list = ups.filter((u) => shopFilter === "all" || catOf(u.key) === shopFilter);
  if (shopSort === "cost") list = [...list].sort((a, b) => a.cost_n - b.cost_n);
  else if (shopSort === "afford") list = [...list].sort((a, b) => (b.affordable - a.affordable) || (a.cost_n - b.cost_n));
  const grouped = shopFilter === "all" && shopSort === "category";
  const body = grouped
    ? cats.map((c) => `<div class="shop-group" style="--chip:${CATS[c].color}"><div class="shop-group-head"><i class="chip-dot"></i>${CATS[c].label}</div><ul class="shop-items">${ups.filter((u) => catOf(u.key) === c).sort((a, b) => srank(a) - srank(b)).map((u) => shopRow(u)).join("")}</ul></div>`).join("")
    : `<ul class="shop-items">${list.map((u) => shopRow(u)).join("")}</ul>`;
  html += `<div class="shop-controls"><div class="shop-filters">${chip("all")}${cats.map(chip).join("")}</div><button class="shop-sort" data-sort-cycle title="Cycle the sort order">sort: <b>${SORT_LABEL[shopSort]}</b></button></div>${body}`;
  return html;
}

// The three stances, as rich pick-cards — the biggest choice each run deserves
// a real explanation, not a one-word button.
const STANCE_INFO = {
  grind: {
    name: "Grind", tag: "active · sustained", line: "Heads-down rapid pushing.",
    points: [
      "<b>2× momentum cap</b> that <b>holds</b> while you keep acting",
      "Every push carries your full momentum as bonus effort",
      "Iron Hold: +30%/rank while momentum is maxed",
    ],
  },
  lurch: {
    name: "Lurch", tag: "active · burst", line: "Wait, then unleash.",
    points: [
      "The <b>Coil</b> starts full and drains — Heave slams on how <b>empty</b> it is",
      "Empty Coil = a massive spike (×13+ at full drain)",
      "Push is just filler; the Heave is the star",
    ],
  },
  patience: {
    name: "Patience", tag: "idle · hands-off", line: "Set it and walk away.",
    points: [
      "No manual push — <b>momentum</b> fills +1 per cycle (a slow battery)",
      "That momentum powers auto-push <b>and</b> auto-heave",
      "Weak at first, effortless forever",
    ],
  },
};
function stanceCards() {
  return `<div class="stance-cards">${["grind", "lurch", "patience"].map((s) => {
    const i = STANCE_INFO[s];
    return `<button class="stance-card stance-${s}" data-act="prestige" data-stance="${s}">
      <div class="stance-card-head"><span class="stance-card-name">${i.name}</span><span class="stance-card-tag">${i.tag}</span></div>
      <div class="stance-card-line">${i.line}</div>
      <ul class="stance-card-points">${i.points.map((p) => `<li>${p}</li>`).join("")}</ul>
      <div class="stance-card-go">choose ${i.name} &rarr;</div>
    </button>`;
  }).join("")}</div>`;
}

// Defiance progress: a bar filling from this point's threshold to the next.
// defiance = ⌊√(rb/10)⌋, so point g sits at g²·10 roll-backs.
function defProgress(snap) {
  const rb = Math.floor(snap.run.rollbacks_n || 0);
  const g = Math.floor(Math.sqrt(rb / 10));      // defiance you'd bank now
  const prev = g * g * 10;                         // roll-backs where this point landed
  const next = snap.next_defiance_at;              // (g+1)²·10
  const pct = next > prev ? Math.max(0, Math.min(100, ((rb - prev) / (next - prev)) * 100)) : 0;
  const need = Math.max(0, Math.ceil(next - rb));
  return `<div class="def-prog">
    <div class="def-prog-head"><span>banking <b>+${g}</b> ⟡ now</span><span class="def-prog-next">next <b>+${g + 1}</b> ⟡ at ${next}</span></div>
    <div class="def-prog-track"><div class="def-prog-fill" style="width:${pct.toFixed(1)}%"></div><span class="def-prog-marks">${rb} / ${next} roll-backs</span></div>
    <div class="def-prog-foot"><b>${need}</b> more roll-back${need === 1 ? "" : "s"} → <b>+1</b> defiance</div>
  </div>`;
}

function defiancePanel(snap) {
  const subs = [
    ["prestige", "Prestige", snap.can_prestige ? "•" : ""],
    ["condition", "Condition", snap.global_skills.length],
    ["pusher", "Pusher", snap.pusher_skills.length],
    ["place", "Place", snap.universe_skills.length],
    ["phases", "Phases", ""],
  ];
  const cards = (nodes, act, attrs) => `<div class="skill-cards">${nodes.map((n) => skillCard(n, act, attrs(n))).join("")}</div>`;
  let html = `<div class="shop-head"><div class="shop-wallet">defiance <b class="v-defiance">${snap.defiance}</b> ⟡${snap.can_prestige ? `<span class="shop-afford">prestige ready · +${snap.prestige_gain}</span>` : ""}</div>${subtabBar("defiance", defianceSub, subs)}</div>`;

  if (defianceSub === "prestige") {
    html += defProgress(snap);
    html += snap.can_prestige
      ? `<div class="prestige-card"><div class="card-top"><b class="card-name">Prestige</b><span class="card-rank">+${snap.prestige_gain} ⟡</span></div><p class="card-desc">End this run and bank <b>+${snap.prestige_gain} defiance</b> — your skill trees stay. Pick the stance for your next run:</p>${stanceCards()}</div>`
      : `<p class="panel-hint">Each roll-back inches the bar toward your next defiance point — keep the overshoot loop going, then prestige here.</p>`;
  } else if (defianceSub === "condition") {
    html += `<p class="sub-hint">The absurd condition itself — applies to every pusher, every universe.</p>${cards(snap.global_skills, "buy-skill", (n) => `data-tree="global" data-key="${n.key}"`)}`;
  } else if (defianceSub === "pusher") {
    html += `<p class="sub-hint">${snap.run.pusher_name} — travels with the character across universes.</p>${cards(snap.pusher_skills, "buy-skill", (n) => `data-tree="pusher" data-key="${n.key}"`)}`;
  } else if (defianceSub === "place") {
    html += `<p class="sub-hint">${snap.run.universe_name} — applies to whoever pushes here.</p>${cards(snap.universe_skills, "buy-skill", (n) => `data-tree="universe" data-key="${n.key}"`)}`;
  } else if (defianceSub === "phases") {
    html += `<p class="sub-hint">Bonuses that fire only during their time of day.</p>`;
    for (const t of snap.phase_tracks) {
      html += `<div class="phase-group ${t.active ? "active" : ""}"><div class="phase-group-head">${t.phase_name}${t.active ? " · now" : ""}</div>${cards(t.nodes, "buy-phase", (n) => `data-phase="${t.phase_key}" data-key="${n.key}"`)}</div>`;
    }
  }
  return html;
}

function universesPanel() {
  return `<p class="panel-hint">Locked. Spend defiance to open a new universe — a different setting for the same loop. Coming soon.</p>`;
}

function pushTip(snap) {
  if (snap.run.manual_disabled) return "Patience runs itself; no manual pushing this run.";
  if (snap.run.stance === "lurch")
    return "Push for Lurch — spends your momentum (adds it as a flat bonus). Strong right after a Heave (full Coil), fading as it drains. Spam it (hold / Space) between heaves — a push never refills the Coil.";
  return "Press and hold (or hold Space) — the rhythm speeds up as you sustain it. Each push adds your momentum as bonus effort.";
}

// ── render ────────────────────────────────────────────────────────────────────
function renderTabs(snap) {
  const shown = [];
  let lockedShown = false;
  for (const t of snap.tabs) {
    if (t.unlocked) shown.push(t);
    else if (!lockedShown) { shown.push(t); lockedShown = true; }
    else break;
  }
  if (!shown.some((t) => t.key === activeTab && t.unlocked)) activeTab = "climb";
  setHTML("tabs", shown
    .map((t) =>
      t.unlocked
        ? `<button class="tab ${t.key === activeTab ? "active" : ""}" data-tab="${t.key}">${t.name}</button>`
        : `<button class="tab locked" data-locked="1" title="${t.hint}">${t.name} ▸</button>`
    )
    .join(""));
}

function render() {
  const snap = JSON.parse(game.snapshot());
  const r = snap.run;
  _pushRateMax = r.push_rate_max || 5; // Speed raises the hold-to-push ceiling
  detectUnlocks(snap);

  // One float per event, fired when its counter rises. The counters that live on
  // the RUN (cascade_seq, rollbacks) reset to 0 at prestige, so re-seed whenever a
  // counter DROPS — otherwise the float stays dead until it climbs past the old run.
  if (_lastAction === null || snap.action_count < _lastAction) _lastAction = snap.action_count;
  else if (snap.action_count > _lastAction + 0.5) { _lastAction = snap.action_count; spawnFloat(snap); }
  // Cascade / Patience auto-heave bursts get their own auto-flavoured float
  if (_lastCascade === null || snap.cascade_seq < _lastCascade) _lastCascade = snap.cascade_seq;
  else if (snap.cascade_seq > _lastCascade + 0.5) { _lastCascade = snap.cascade_seq; spawnCascadeFloat(snap); }
  // a roll-back banked scorn → crit-float on the scorn counter
  if (_lastRollback === null || snap.run.rollbacks_n < _lastRollback) _lastRollback = snap.run.rollbacks_n;
  else if (snap.run.rollbacks_n > _lastRollback + 0.5) { _lastRollback = snap.run.rollbacks_n; spawnScornFloat(snap); }

  const f = snap.day_fraction;
  const nOp = nightOp(f);
  // pick a random moon phase once per night, when night first begins
  if (nOp > 0.001 && !moonActive) { moonActive = true; currentMoon = MOON_PHASES[Math.floor(Math.random() * MOON_PHASES.length)]; }
  else if (nOp <= 0.001) moonActive = false;
  setHTML("scene", buildScene(r.progress, f, r.rollbacks_n));
  $("scene").style.background = `linear-gradient(180deg, ${skyColor(f)} 0%, #0a0c14 58%, #060810 100%)`;
  $("scene").style.setProperty("--sun-op", sunOp(f).toFixed(2));
  $("scene").style.setProperty("--moon-op", nOp.toFixed(2));
  // day-cycle countdown + a lightweight bar that fills toward the next phase
  const within = (((f * 6) % 1) + 1) % 1;                 // 0→1 through this phase
  const remain = Math.ceil((1 - within) * PHASE_SECONDS); // seconds left in phase
  const cd = `${Math.floor(remain / 60)}:${String(remain % 60).padStart(2, "0")}`;
  // Build the report shell ONCE, then update text in place — rebuilding innerHTML
  // every frame recreated the stance button and made its :hover flicker.
  const report = $("report");
  if (!report.dataset.built) {
    report.dataset.built = "1";
    report.innerHTML = `<span id="rp-phase"></span> <span class="report-cd" id="rp-cd"></span> · <span id="rp-stance"></span>`;
  }
  $("rp-phase").textContent = snap.phase;
  $("rp-cd").textContent = cd;
  const stanceKey = r.stance || "";
  if (report.dataset.stance !== stanceKey) {
    report.dataset.stance = stanceKey;
    $("rp-stance").innerHTML = r.stance
      ? `<button class="report-stance" data-goto-stance title="Open the ${r.stance} shop">${r.stance}</button>`
      : "no stance";
  }
  const pbFill = $("phase-bar").firstElementChild;
  if (pbFill) pbFill.style.width = `${(within * 100).toFixed(1)}%`;

  // primary actions — Push + Heave (Roll back gets its own row, below)
  // Lurch flips the emphasis: Heave is the star (big), Push is small filler.
  const lurch = r.stance === "lurch";
  $("acts").classList.toggle("lurch", lurch);
  let acts = `<button class="act-push" id="b-push" data-act="push" ${r.manual_disabled ? "disabled" : ""} title="${pushTip(snap)}">${r.action_verb}</button>`;
  // Active stances: a clickable Heave. Patience: the SAME cooldown bar, but it
  // fires itself (auto-heave) — shown so you can watch the slam recharge, not
  // hidden. No data-act, so it's non-interactive and the drag gesture ignores it.
  const heaveTip = lurch
    ? "Lurch's charge-and-slam. Fire it at an EMPTY Coil for the biggest hit (scales with how drained it was), and it re-coils momentum to FULL — then spam pushes to spend that momentum before it drains. Only the Heave refills the Coil."
    : "A bigger push that adds momentum ×2 as effort. Fills as it recharges; tap when full.";
  acts += r.manual_disabled
    ? `<button class="act-heave act-auto" id="b-heave" disabled title="Patience auto-Heaves on this bar — a big slam powered by your momentum. It fires itself when full; you never click it.">Auto-Heave</button>`
    : `<button class="act-heave" id="b-heave" data-act="heave" title="${heaveTip}">Heave</button>`;
  setHTML("acts", acts);
  // The Heave button IS its cooldown bar: update the fill by style so the
  // element persists (no re-render flash), giving a smooth recharge sweep.
  const hb = $("b-heave");
  if (hb) {
    if (r.heave_ready) {
      hb.style.background = "rgba(126, 236, 181, 0.14)";
      hb.disabled = r.manual_disabled; // Patience: shown but never clickable
    } else {
      const p = Math.round(r.heave_cooldown * 100);
      hb.style.background = `linear-gradient(90deg, var(--phosphor-dim) ${p}%, var(--bg-elev) ${p}%)`;
      hb.disabled = true;
    }
  }

  // Roll back — at the summit a countdown runs while you overshoot; it auto-banks
  // at 0, or "Bank now" ends it early. Shell built once so the countdown text
  // updates smoothly (no per-frame button recreation / hover flash).
  const rr = $("rollrow");
  if (r.can_rollback) {
    if (rr.dataset.on !== "1") {
      rr.dataset.on = "1";
      rr.innerHTML = `<div class="roll-panel"><span class="roll-info">auto roll-back in <b id="roll-cd">5.0s</b> · banking <b id="roll-mult">×1.00</b> scorn — climb higher!</span><button class="act-roll" data-act="rollback" title="Bank now — end the countdown early for a faster next climb (you keep the current overshoot).">Bank now</button></div>`;
    }
    const cdEl = $("roll-cd"), mEl = $("roll-mult");
    if (cdEl) cdEl.textContent = `${Math.max(0, r.rollback_countdown).toFixed(1)}s`;
    if (mEl) mEl.textContent = `×${r.overshoot_mult.toFixed(2)}`;
  } else if (rr.dataset.on !== "0") {
    rr.dataset.on = "0";
    rr.innerHTML = "";
  }

  // momentum bar — always shown when a stance has momentum. It's always "momentum"
  // (Patience's just fills +1/cycle instead of per push), only the note differs.
  // Shell is rebuilt only when the note/cap changes so the fill stays smooth.
  const mb = $("mbar");
  if (r.momentum_max <= 0) {
    mb.hidden = true;
  } else {
    mb.hidden = false;
    const label = "momentum";
    const note = r.manual_disabled
      ? "fills +1/cycle · powers auto-push &amp; auto-heave"
      : r.stance === "lurch"
      ? "pushes spend it · the emptier it is, the bigger your Heave (which refills it)"
      : "added to every push";
    const sig = `${note}|${Math.round(r.momentum_max)}`;
    if (mb.dataset.sig !== sig) {
      mb.dataset.sig = sig;
      setHTML("mbar", `<span class="mbar-label">${label} <b id="mbar-num">0</b> / ${Math.round(r.momentum_max)} <em>&mdash; ${note}</em></span><span class="mbar-track"><span class="mbar-fill" id="mbar-fill"></span></span>`);
    }
    const num = $("mbar-num");
    const fill = $("mbar-fill");
    if (num) num.textContent = Math.floor(r.momentum);
    if (fill) fill.style.width = Math.round((r.momentum / r.momentum_max) * 100) + "%";
  }

  // auto-push has its own card: an unlock-progress meter, then a running state
  const ac = $("autocard");
  if (r.auto_unlocked) {
    ac.classList.add("on");
    const ivl = (+r.auto_push_interval).toFixed(2).replace(/\.?0+$/, "");
    setHTML("autocard", `<span class="autocard-icon">⚙</span><span class="autocard-main"><b>Auto-push running</b> · +${r.auto_push_size} every ${ivl}s <em>(${r.auto_push_rate}/s)</em> — the boulder rolls itself</span>`);
  } else if (!r.manual_disabled) {
    ac.classList.remove("on");
    const pct = Math.min(100, (r.pushes_this_run / 5) * 100);
    setHTML("autocard", `<span class="autocard-icon">⚙</span><span class="autocard-main"><b>Auto-push</b> — push <b>${r.pushes_this_run}/5</b> to unlock it, free<span class="autocard-bar"><span style="width:${pct}%"></span></span></span>`);
  } else {
    setHTML("autocard", "");
  }

  // progress + chips
  const pct = Math.round(r.progress * 100);
  let read = `<div class="climb-row"><span class="climb-label">${r.height} / ${r.summit}${r.at_summit ? ` · <b>SUMMIT</b>` : ""}</span><span class="climb-pct">${pct}%</span></div><div class="climbbar"><div class="climbbar-fill" style="width:${pct}%"></div></div>`;
  read += `<div class="chips"><span>scorn <b class="v-scorn">${r.scorn}</b></span>`;
  if (tab(snap, "defiance").unlocked) read += `<span>defiance <b class="v-defiance">${snap.defiance}</b></span>`;
  read += `<span>roll-backs <b>${r.rollbacks}</b></span></div>`;
  setHTML("readout", read);

  renderTabs(snap);

  setHTML(
    "panel",
    activeTab === "scorn" ? scornPanel(snap) :
    activeTab === "defiance" ? defiancePanel(snap) :
    activeTab === "universes" ? universesPanel() :
    climbPanel(snap)
  );

  // A stat chip the popover was anchored to may be gone after a re-render.
  if (_popAnchor && !_popAnchor.isConnected) hidePop();
}

// ── input (delegated) ─────────────────────────────────────────────────────────
$("console").addEventListener("click", (e) => {
  const t = e.target.closest("[data-tab]");
  if (t) { activeTab = t.dataset.tab; render(); return; }
  const gs = e.target.closest("[data-goto-stance]");
  if (gs) { activeTab = "scorn"; scornSub = "stance"; render(); return; }
  const sub = e.target.closest("[data-subtab]");
  if (sub) {
    const [scope, key] = sub.dataset.subtab.split(":");
    if (scope === "defiance") defianceSub = key; else scornSub = key;
    render(); return;
  }
  const fil = e.target.closest("[data-filter]");
  if (fil) { shopFilter = fil.dataset.filter; render(); return; }
  const srt = e.target.closest("[data-sort-cycle]");
  if (srt) { shopSort = SORTS[(SORTS.indexOf(shopSort) + 1) % SORTS.length]; render(); return; }
  const b = e.target.closest("button[data-act]");
  if (!b) return;
  const a = b.dataset.act;
  // push is hold-driven (see startHold below), not click-driven
  if (a === "heave") { stopHold(); send({ kind: "heave" }); } // let go of the boulder to slam it
  else if (a === "rollback") send({ kind: "rollback" });
  else if (a === "buy-upgrade") send({ kind: "buy_upgrade", key: b.dataset.key });
  else if (a === "buy-branch") send({ kind: "buy_branch", key: b.dataset.key });
  else if (a === "choose-subbuild") send({ kind: "choose_sub_build", key: b.dataset.key });
  else if (a === "buy-skill") send({ kind: "buy_skill", tree: b.dataset.tree, key: b.dataset.key });
  else if (a === "buy-phase") send({ kind: "buy_phase_skill", phase: b.dataset.phase, key: b.dataset.key });
  else if (a === "prestige") {
    const snap = JSON.parse(game.snapshot());
    send({ kind: "prestige", universe: snap.run.universe, stance: b.dataset.stance });
  }
});

// hold-to-push wiring: pointer-hold on the push button, plus Space. Release is
// window-level so it survives the button re-rendering mid-hold and dragging off.
// Start the drag gesture only on the Push button; the pointer's region then
// drives push/heave until release (release is window-level so it always lands).
$("console").addEventListener("pointerdown", (e) => {
  if (e.target.closest('button[data-act="push"]')) {
    e.preventDefault();
    ptrX = e.clientX; ptrY = e.clientY;
    heldDown = true; region = null;
    holdStart = performance.now(); pausedAt = 0; // fresh gesture → ramp starts at 2/s
    setRegion("push");
  }
});
window.addEventListener("pointermove", (e) => {
  if (!heldDown) return;
  ptrX = e.clientX; ptrY = e.clientY;
  setRegion(regionAt(ptrX, ptrY));
});
window.addEventListener("pointerup", endGesture);
window.addEventListener("pointercancel", endGesture);
document.addEventListener("keydown", (e) => {
  if (e.code !== "Space") return;
  const el = document.activeElement;
  if (el && (el.tagName === "INPUT" || el.tagName === "TEXTAREA")) return;
  e.preventDefault(); // block scroll on EVERY Space, incl. auto-repeat while held
  if (!e.repeat) { holdStart = performance.now(); pausedAt = 0; startHold(); } // fresh press → ramp from 2/s
});
document.addEventListener("keyup", (e) => { if (e.code === "Space") stopHold(); });

// ── stat detail popover ───────────────────────────────────────────────────────
// A themed replacement for the native `title` tooltip: one reused card, anchored
// to whichever `.has-detail` chip you hover, tap, or focus. Long text scrolls
// inside the card (short details never do). See CSS `.stat-pop`.
const _pop = document.createElement("div");
_pop.className = "stat-pop";
_pop.setAttribute("role", "tooltip");
_pop.hidden = true;
_pop.innerHTML = `<span class="stat-pop-head"></span><span class="stat-pop-body"></span>`;
document.body.appendChild(_pop);
const _popHead = _pop.querySelector(".stat-pop-head");
const _popBody = _pop.querySelector(".stat-pop-body");
let _popAnchor = null;
let _lastPT = "mouse"; // pointerType of the last press, so touch can toggle

function positionPop(a) {
  const r = a.getBoundingClientRect();
  const pw = _pop.offsetWidth, ph = _pop.offsetHeight, gap = 8, edge = 8;
  let left = r.left + r.width / 2 - pw / 2;
  left = Math.max(edge, Math.min(left, window.innerWidth - pw - edge));
  let top = r.top - ph - gap, place = "top";
  if (top < edge) { top = r.bottom + gap; place = "bottom"; } // flip below when it'd clip the top
  _pop.style.left = `${left + window.scrollX}px`;
  _pop.style.top = `${top + window.scrollY}px`;
  _pop.dataset.place = place;
  // point the arrow at the chip's centre, clamped inside the card
  const cx = r.left + r.width / 2 - left;
  _pop.style.setProperty("--arrow-x", `${Math.max(14, Math.min(cx, pw - 14))}px`);
}

let _hideTimer = null;
function showPop(a) {
  const detail = a.dataset.detail;
  if (!detail) return;
  clearTimeout(_hideTimer); // cancel a pending hide (e.g. from a live re-render)
  _popHead.textContent = a.querySelector(".stat-label, .record-label")?.textContent || "";
  _popBody.textContent = detail;
  _popBody.scrollTop = 0;
  _pop.hidden = false;
  if (_popAnchor && _popAnchor !== a) _popAnchor.classList.remove("pop-open");
  _popAnchor = a;
  a.classList.add("pop-open");
  positionPop(a);
}

function hidePop() {
  clearTimeout(_hideTimer);
  if (!_popAnchor) return;
  _pop.hidden = true;
  _popAnchor.classList.remove("pop-open");
  _popAnchor = null;
}

// Debounced hide: a live stat (e.g. Lurch's ticking heave ×N) re-renders the chip
// each frame, firing pointerout→pointerover. Delay the hide a hair; if the pointer
// is still over an equivalent chip (the re-rendered one), re-anchor instead of
// hiding — so the popover no longer flickers on updating chips.
function scheduleHide() {
  clearTimeout(_hideTimer);
  _hideTimer = setTimeout(() => {
    const el = document.elementFromPoint(_ptrX, _ptrY);
    const stat = el && el.closest && el.closest("[data-detail]");
    if (stat) showPop(stat); else hidePop();
  }, 60);
}

let _ptrX = 0, _ptrY = 0; // last pointer position, for the re-anchor fallback
document.addEventListener("pointermove", (e) => { _ptrX = e.clientX; _ptrY = e.clientY; }, true);
document.addEventListener("pointerdown", (e) => { _lastPT = e.pointerType || "mouse"; }, true);
document.addEventListener("pointerover", (e) => {
  if (_lastPT === "touch") return; // touch is handled by click-to-toggle
  const stat = e.target.closest?.("[data-detail]");
  if (stat) showPop(stat);
});
document.addEventListener("pointerout", (e) => {
  if (!_popAnchor || _lastPT === "touch") return;
  const stat = e.target.closest?.("[data-detail]");
  // ignore moves between the chip's own children; only a real leave hides (debounced)
  if (stat === _popAnchor && !_popAnchor.contains(e.relatedTarget)) scheduleHide();
});
document.addEventListener("click", (e) => {
  const stat = e.target.closest?.("[data-detail]");
  if (_lastPT === "touch") {
    if (stat) { _popAnchor === stat ? hidePop() : showPop(stat); return; }
    hidePop();
  } else if (!stat) hidePop(); // mouse: an outside click dismisses
});
document.addEventListener("focusin", (e) => {
  const stat = e.target.closest?.("[data-detail]");
  if (stat) showPop(stat); else if (!_pop.contains(e.target)) hidePop();
});
document.addEventListener("keydown", (e) => { if (e.key === "Escape") hidePop(); });
// capture-phase to catch scrolling containers, but not the popover's own body
window.addEventListener("scroll", (e) => { if (!_pop.contains(e.target)) hidePop(); }, true);
window.addEventListener("resize", hidePop);

// ── save / export / import / abandon ──────────────────────────────────────────
function persist() {
  const blob = game.save(Date.now());
  if (blob) localStorage.setItem(SAVE_KEY, blob);
}
$("btn-save").addEventListener("click", persist);
setInterval(persist, 15000);
document.addEventListener("visibilitychange", () => { if (document.hidden) persist(); });
window.addEventListener("beforeunload", persist);

$("btn-export").addEventListener("click", () => {
  const blob = game.export(Date.now());
  if (!blob) return;
  const url = URL.createObjectURL(new Blob([blob], { type: "application/json" }));
  const a = document.createElement("a");
  a.href = url; a.download = "one-must-imagine.save.json"; a.click();
  URL.revokeObjectURL(url);
});
$("file-import").addEventListener("change", async (e) => {
  const file = e.target.files[0];
  if (!file) return;
  const rep = JSON.parse(game.import(await file.text(), Date.now()));
  notice(
    rep.outcome === "resumed" ? "Save imported." :
    rep.outcome === "refused" ? `Import refused: that save is from a newer version (v${rep.found}). Your run is untouched.` :
    `Import failed: ${rep.message || "unreadable"}. Your run is untouched.`,
    rep.outcome !== "resumed"
  );
  e.target.value = "";
  _lastAction = null; _lastCascade = null; _lastRollback = null; // re-seed against the imported game
  render();
});
$("btn-new").addEventListener("click", () => {
  if (!confirm("Abandon this run and start over? The boulder waits.")) return;
  game.new_game();
  localStorage.removeItem(SAVE_KEY);
  const n = $("boulder-notice"); if (n) n.hidden = true;
  activeTab = "climb";
  _lastAction = null; _lastCascade = null; _lastRollback = null; // re-seed the float counters against the fresh game
  render();
});

// ── loop: tick every frame, render + animate on calmer cadences ───────────────
let last = performance.now();
let sinceRender = 0;
let sinceFrame = 0;
function loop(now) {
  const dt = now - last;
  last = now;
  game.tick(dt);
  updateClouds(dt);
  sinceRender += dt;
  sinceFrame += dt;
  if (sinceFrame >= FRAME_MS) { frame++; sinceFrame = 0; }
  if (sinceRender >= RENDER_MS) { sinceRender = 0; render(); }
  requestAnimationFrame(loop);
}
render();
requestAnimationFrame(loop);
