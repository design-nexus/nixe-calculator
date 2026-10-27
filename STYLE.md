# Nexus App Style

For **Nyx Suite** apps, read [`nyx/STYLE.md`](nyx/STYLE.md), which is the current
authoritative guide, and inspect Music's actual `src/window.rs` before writing UI.
Its required GTK sidebar construction applies to every Nyx app: explicitly set
`hexpand(false)` on the sidebar, navigation list and navigation scroller, give only
the page column `hexpand(true)`, and use the shared expanded/compact CSS widths.
A width request is a minimum, not a cap; it does not prevent a sidebar from taking
half the window. Copy the existing resize behavior rather than recreating it.

How my apps look and behave. The reference implementation is **Music**
(`~/Projects/nexus-music`, especially `src/style.css`, `src/window.rs` and
`src/settings_dialog.rs`). **Settings** (`~/Projects/settings`) is the reference for
an app whose pages are all settings. When building a new app, match this document. If
a detail isn't covered here, copy what Music does.

---

## 1. Principles

- **One flat surface.** The whole window is a single `bg` surface: top bar, sidebar,
  page and status bar alike. Areas are separated only by 1 px hairlines, never by
  different fills or shadows.
- **Monospace throughout.** Every label, button and readout uses JetBrains Mono.
- **Calm and dense.** Low contrast, one accent used sparingly for state (where you
  are, on, selected, the page name in the top bar). No glows, no big titles.
- **Everything comes from semantic color tokens.** Never hard-code a color in a
  component. Every color is one of the tokens below, or one of the derived mixes.
- **Themeable, and follows Omarchy by default.** Every app ships the bundled themes,
  accepts user themes, and has a *Follow Omarchy* mode that restyles live.
- **Native, keyboard-first, no client-side chrome.** No title bar (Hyprland manages
  windows); the app draws its own top bar. <kbd>Ctrl</kbd>+<kbd>F</kbd> searches,
  <kbd>Esc</kbd> clears or closes, <kbd>Ctrl</kbd>+<kbd>B</kbd> collapses the sidebar,
  <kbd>F1</kbd> lists the shortcuts, <kbd>Ctrl</kbd>+<kbd>Q</kbd>/<kbd>W</kbd> closes.
- **Works tiled at half-screen.** Layouts must fit an 800 px wide window with no
  horizontal scroll (see Responsive).

---

## 2. Color tokens

Every theme defines exactly these 11 colors, plus a `light` flag. `on_accent` is
derived: text drawn on an accent fill.

| Token | Role |
| --- | --- |
| `bg` | Window and content background |
| `surface` | Popovers and toasts: one step off `bg` (the window itself is all `bg`) |
| `text` | Primary text |
| `dim_text` | Secondary text: descriptions, group headings, readouts |
| `accent` | The page name in the top bar, focus, "on" switches, sort arrows, slider fill; mixed into `@nav_active` for selection |
| `border` | Base of `@hairline`: every line between areas and every card outline |
| `muted` | Neutral fills: slider troughs, off switches, the settings card's `@lift` |
| `highlight` | A spare selection tone (hover and selection use `@row_hover` and `@nav_active`) |
| `danger` | Destructive actions, errors, warnings |
| `glow` | Kept for themes and the Glow preference (usually = accent); the flat look draws no glows |
| `shadow` | Drop shadows, code-block backgrounds (darkest color) |
| `on_accent` | Derived: `#ffffff` for light themes, otherwise `bg` |

Names in code: GTK `@theme_bg`, `@theme_dim_text`, …; CSS/web `--nx-bg`,
`--nx-dim-text`, …; QML/Rust `palette.bg`, `palette.dimText`/`dim_text`.

### Derived colors

Define these once at the top of the stylesheet, loaded in the same provider as the
palette, and use them by name:

```css
@define-color hairline mix(@theme_bg, @theme_border, 0.82);
@define-color divider alpha(@theme_text, 0.08);
@define-color nav_active mix(@theme_bg, @theme_accent, 0.18);
@define-color row_hover mix(@theme_bg, @theme_accent, 0.07);
@define-color lift mix(@theme_bg, @theme_muted, 0.09);
@define-color lift_active mix(@lift, @theme_accent, 0.1);
@define-color veil alpha(mix(@theme_bg, @theme_shadow, 0.45), 0.82);
```

| Use | Recipe |
| --- | --- |
| Every line between areas, card and input borders | `@hairline` |
| Divider between rows inside a card | `@divider` |
| Active nav item, selected row, selected chip or segment, primary button | `@nav_active` |
| Hover on rows, nav items and flat buttons | `@row_hover` |
| Settings card's right column | `@lift` |
| Active item in the settings card's list | `@lift_active` |
| Dimmed window behind a card | `@veil` |
| Breadcrumb's page name, sort arrows, on-switch knob, icons on primary buttons | `accent` |
| Secondary text: descriptions, headings, readouts, status bar | `dim_text` |
| Danger: armed buttons, warnings, errors | `danger` (fills at 0.10–0.18 alpha) |

---

## 3. Themes

### Bundled themes (ship all of them)

| id | Name | bg | surface | text | dim_text | accent | border | muted | highlight | danger | glow | shadow | light |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| dracula | Dracula | #282a36 | #21222c | #f8f8f2 | #6272a4 | #bd93f9 | #44475a | #44475a | #343746 | #ff5555 | #bd93f9 | #191a21 | no |
| catppuccin-mocha | Catppuccin Mocha | #1e1e2e | #181825 | #cdd6f4 | #7f849c | #89b4fa | #313244 | #45475a | #313244 | #f38ba8 | #89b4fa | #11111b | no |
| catppuccin-macchiato | Catppuccin Macchiato | #24273a | #1e2030 | #cad3f5 | #8087a2 | #8aadf4 | #363a4f | #494d64 | #363a4f | #ed8796 | #8aadf4 | #181926 | no |
| catppuccin-frappe | Catppuccin Frappé | #303446 | #292c3c | #c6d0f5 | #838ba7 | #8caaee | #414559 | #51576d | #414559 | #e78284 | #8caaee | #232634 | no |
| catppuccin-latte | Catppuccin Latte | #eff1f5 | #e6e9ef | #4c4f69 | #8c8fa1 | #1e66f5 | #ccd0da | #bcc0cc | #dce0e8 | #d20f39 | #1e66f5 | #9ca0b0 | yes |
| tokyo-night | Tokyo Night | #1a1b26 | #16161e | #c0caf5 | #565f89 | #7aa2f7 | #292e42 | #3b4261 | #283457 | #f7768e | #7aa2f7 | #0f0f14 | no |
| tokyo-night-storm | Tokyo Night Storm | #24283b | #1f2335 | #c0caf5 | #565f89 | #7aa2f7 | #292e42 | #3b4261 | #2e3c64 | #f7768e | #7aa2f7 | #1b1e2d | no |
| tokyo-night-moon | Tokyo Night Moon | #222436 | #1e2030 | #c8d3f5 | #636da6 | #82aaff | #2f334d | #3b4261 | #2d3f76 | #ff757f | #82aaff | #191a2a | no |
| one-dark-pro | One Dark Pro | #282c34 | #21252b | #abb2bf | #5c6370 | #61afef | #3e4451 | #4b5263 | #2c313a | #e06c75 | #61afef | #181a1f | no |
| nord | Nord | #2e3440 | #292e39 | #eceff4 | #7b88a1 | #88c0d0 | #3b4252 | #434c5e | #3b4252 | #bf616a | #88c0d0 | #242933 | no |
| gruvbox-dark | Gruvbox Dark | #282828 | #1d2021 | #ebdbb2 | #928374 | #fabd2f | #3c3836 | #504945 | #3c3836 | #fb4934 | #fabd2f | #141414 | no |
| rose-pine | Rosé Pine | #191724 | #1f1d2e | #e0def4 | #6e6a86 | #c4a7e7 | #26233a | #403d52 | #26233a | #eb6f92 | #c4a7e7 | #111019 | no |
| everforest | Everforest | #2d353b | #272e33 | #d3c6aa | #859289 | #a7c080 | #3d484d | #475258 | #3d484d | #e67e80 | #a7c080 | #1e2326 | no |
| solarized-dark | Solarized Dark | #002b36 | #00232c | #93a1a1 | #586e75 | #268bd2 | #073642 | #0f4a56 | #073642 | #dc322f | #268bd2 | #001e26 | no |
| kanagawa | Kanagawa | #1f1f28 | #16161d | #dcd7ba | #727169 | #7e9cd8 | #2a2a37 | #363646 | #2d4f67 | #e82424 | #7e9cd8 | #0d0c0c | no |

The fallback theme, when nothing else is available, is **Tokyo Night**.

### Follow Omarchy (the default mode)

Other apps (GTK, libadwaita, Qt, KDE) get the theme from hyprchroma, which
Settings → Theme → *Apps follow the theme* controls. New apps don't need to
theme anything outside their own window.

Read `~/.local/state/omarchy/current/theme/colors.toml` (respect `$XDG_STATE_HOME`)
and map it:

| Token | Omarchy key (fallbacks in order) |
| --- | --- |
| bg | `background` (required) |
| surface | `dark_background` → `background` |
| text | `foreground` (required) |
| dim_text | `dark_foreground` → `muted` → `foreground` |
| accent | `accent` → `blue` (required) |
| border | `lighter_background` → `selection` → `foreground` |
| muted | `muted` → `lighter_background` → `background` |
| highlight | `selection` → `lighter_background` → `background` |
| danger | `red` → `#e06c75` |
| glow | = accent |
| shadow | `darker_background` → `#000000` |
| light | `mode == "light"` |

- **Live updates:** poll once a second for changes to the modification time of
  `colors.toml` or the contents of `…/current/theme.name`, and restyle in place. Polling
  is more reliable than inotify, because Omarchy swaps the whole directory.
- If `colors.toml` is missing or invalid, hide the *Follow Omarchy* switch and use the
  chosen bundled theme.

### User themes

Each user theme is one file in `~/.config/<app>/themes/<id>.toml`, with the same keys
as a bundled theme plus a display name:

```toml
name = "My Theme"
bg = "#1a1b26"
surface = "#16161e"
text = "#c0caf5"
dim_text = "#565f89"
accent = "#7aa2f7"
border = "#292e42"
muted = "#3b4261"
highlight = "#283457"
danger = "#f7768e"
glow = "#7aa2f7"
shadow = "#0f0f14"
light = false
```

List user themes after the bundled ones, sorted by name, with ids `custom:<file stem>`.

### Preferences (per app, `~/.config/<app>/settings.toml`)

`mode = "omarchy" | "theme"`, `theme = "<id>"`, `glow = true`,
`reduce_motion = false`, `sidebar_collapsed = false`. Picking a theme sets `mode = "theme"`. Every app exposes these
in its own settings, in a group called **<App> window** (e.g. **Settings window**): Follow Omarchy theme, a theme
dropdown (disabled while following), current-color swatches, Glow, and Reduce motion.

---

---

## 4. Typography

- **Font:** `"JetBrains Mono", "JetBrainsMono Nerd Font", monospace` on the window,
  weight 500, with tabular figures where numbers line up. Don't bundle it.
- **Base size:** the toolkit default (≈ 13–14 px). Everything else is relative:

| Element | Size | Weight | Color | Extra |
| --- | --- | --- | --- | --- |
| Top bar path | 1em | 500 | `dim_text`, page name in `accent` | `App / Page`, or `App / Group / Page` |
| Group heading | 0.9em | 600 | `dim_text` | UPPERCASE, letter-spacing 2 px |
| Sidebar group label | 0.76em | 600 | `dim_text` | UPPERCASE, letter-spacing 1 px |
| Settings card heading | 0.92em | 600 | `dim_text` | `SETTINGS`, letter-spacing 2.5 px |
| Card header title | 1.2em | 600 | `text` | |
| Nav item, row title | 1em | 500 / 700 | `text` | |
| Row description | 0.88em | 400 | `dim_text` | wraps |
| Table header | 0.86em | 500 | `dim_text` | |
| Status bar | 0.86em | 500 | `dim_text` | `F1` key in `text` |

There are no page titles or page descriptions: the top bar names the page, and a
page's one-line description (if any) goes in the status bar.

GTK CSS can't transform text, so write uppercase headings with `to_uppercase()` in
code, and keep the original title for search.

**Copy style:** sentence case, American spelling. Descriptions are one short sentence
that says what the option *does* for the user, not what it's called internally.

---

## 5. Shape, spacing, motion

| Thing | Value |
| --- | --- |
| Radius scale | **4** small parts (tags, key caps, segment buttons) · **6** buttons, chips, inputs, nav items · **10** cards, banners, toasts · **14** the settings card. Only switches and sliders are pills. |
| Height scale | 30 top-bar buttons · 28 chips · 32 buttons and inputs · 35 table rows · 42 nav items · 46 top bar and toolbars · 34 status bar |
| Spacing | multiples of 2 and 4 |
| Page padding | 12 px top · 24 px sides · 36 px bottom (narrow: 12 / 16 / 32) |
| Group heading margin | 26 px above, 12 px below; the page's first heading has none above, so the gap to the top edge equals the gap below it |
| Pages that fill the window | no padding at all: a toolbar row, then the table or grid edge to edge |
| Row padding | 10 px 20 px, 52 px tall at least |
| Transitions | 150 ms ease-out for hover/color; 160 ms crossfade between pages |
| Reduce motion | all durations 0; toolkit animations off |

---

## 6. Layout

```
┌──────────────────────────────────────────────────────────────┐
│ ▣  App / Page                          [✎]  ⌕   ⚙   ✕       │ top bar, 46 px
├──────────────┬───────────────────────────────────────────────┤
│ ⌂ Home       │  [Filter…]  [Action] [Action]           [⋯]  │ toolbar row, 46 px
│              ├───────────────────────────────────────────────┤
│ LIBRARY      │  Name ↑            │ Size   │ Modified       │ table header, 34 px
│ ▣ Item  ◀──  ├───────────────────────────────────────────────┤
│ ▣ Item       │  row                                          │ rows, 35 px
│──────────────│  row                                          │
│ PINNED       │                                               │
│ ▣ Item       │                                               │
├──────────────┴───────────────────────────────────────────────┤
│ F1 Shortcuts                                    1,204 items │ status bar, 34 px
└──────────────────────────────────────────────────────────────┘
```

- **Window:** `bg` everywhere. No title bar: set an empty widget as the GTK titlebar.
  Default size about 1120 × 800. The frame is a vertical box: top bar, body (sidebar
  and page stack), any app bar (a player bar), status bar. The window's overlay holds
  toasts and cards.
- **Top bar** (`.top-bar`, 46 px, hairline below): the sidebar toggle
  (`sidebar-show-symbolic`; a back arrow in apps without a sidebar), then the path,
  `App / Page` with the page name in `accent` (ellipsized). On the right, flat 30 px
  icon buttons (`.bar-button`): the page's config file when it has one (a menu when it
  has several), search, any app-wide panel (the equalizer), settings (gear), close.
- **Search** lives in the top bar: the search button reveals an entry beside it;
  <kbd>Ctrl</kbd>+<kbd>F</kbd> shows and focuses it; <kbd>Esc</kbd> clears and hides
  it. On a page with its own filter field (a process list), <kbd>Ctrl</kbd>+<kbd>F</kbd>
  goes there instead.
- **Sidebar** (`.settings-navigation`, 250 px, `bg`, hairline on the right): no app
  heading, no search, no footer. Nav items are an icon and a label (and an optional
  count in `dim_text`); hover is `@row_hover`, the active item `@nav_active` with no
  ring. Group labels are small UPPERCASE with a hairline above each group (none above
  the first). A group label can carry a flat `+` at its right (new playlist, category).
  Settings is not in the sidebar.
- **Pages:** a stack, crossfading, built on first visit. Content pages scroll
  vertically with the scrollbar hidden and stay under ~920 px wide, centered.
  Table and grid pages fill the window: a hairline toolbar row on top (controls
  vertically centered, 7 px above and below), then the content edge to edge.
- **Status bar** (`.status-bar`, 34 px, hairline above, 0.86em `dim_text`): a flat
  `F1 Shortcuts` button on the left that opens the shortcuts list; on the right a live
  readout that matters for the app (song count and length, speeds, CPU and memory,
  the open file and page, the page's description in Settings). Optional small toggles
  sit at its right end (slow mode, live/paused).
- **Settings** open as a **card over the window**, not a page (see Components).
  `navigate("settings")` opens it, so old links and `--section settings` still work.
- **Fullscreen** (players, viewers) hides the top bar, sidebar, status bar and app bars.
- **Collapsible sidebar:** the top bar toggle and <kbd>Ctrl</kbd>+<kbd>B</kbd> collapse
  it to icons (about 52 px: labels, group labels and counts hide, tooltips stay, icons
  centered in the 40 px column). The choice is saved as `sidebar_collapsed`. The
  sidebar is compact when *either* the window is under 980 px *or* the user collapsed
  it.
- **Responsive:** nothing may be cut off or force a horizontal scrollbar at 800 px.
  Keep every page's minimum width under about 580 px: titles wrap, dropdowns ellipsize,
  groups of buttons flow onto two lines, toolbar labels drop before icons.

---

## 7. Components

**Group:** an UPPERCASE heading, an optional note in `dim_text`, then **one card holding
all the group's rows** (`.group-list`: hairline border, radius 10, no fill), rows split
by `@divider` hairlines. Content with its own layout (tiles, a grid, an editor) goes in
a group without the card. Name each group's wrapper after its title
(`set_widget_name`), so the settings card can list it.

**Option row (`.settings-option`):** title and description on the left (they expand
and wrap), the control on the right, vertically centered. A row on its own outside a
card draws a hairline border, radius 10. A *stacked* row puts a wide control under its
text.

**Buttons:**
- default: transparent with a hairline border, `@row_hover` on hover;
- primary (`suggested-action`): `@nav_active` fill, no border, weight 600, icon in `accent`;
- destructive: `danger` text on `alpha(danger, 0.12)` with a `danger` border;
- flat and top-bar buttons: no border, `dim_text` icon, `text` on hover;
- buttons on a banner: solid `accent` with `on_accent` text (`danger` on a warning
  banner), so the banner's action stands out on its tint.

Dangerous actions need two clicks: the first arms the button ("Click again to …"),
the second acts; it disarms after 4 s.

**Chip** (filters, + Spacer): a 28 px rectangle, radius 6, hairline border; selected
is `@nav_active` with no border. Not a pill.

**Segmented control:** a hairline track, radius 6, 2 px padding; the selected segment
is `@nav_active` with `text`. Prefer it to a dropdown when every option fits on one line.

**Switch:** a 44 × 26 pill with a hairline border and `alpha(muted, 0.3)` fill; the
18 px knob is `dim_text` when off. On, the track is `mix(bg, accent, 0.25)` and the
knob `accent`.

**Slider:** 4 px trough in `alpha(muted, 0.8)` with an `accent` fill and a 16 px knob.
The value readout sits to the right in tabular figures. No scale marks.

**Table** (`ColumnView`): transparent; header 34 px, labels 0.86em `dim_text`, a hairline
under the header and between header cells, the sort arrow in `accent`. Rows 35 px, no
striping, no row dividers, no rounded corners; hover `@row_hover`, selected
`@nav_active`. Reset the header button's font size to `1em` before sizing its label,
or GTK's own smaller size compounds.

**Settings card** (`settings_dialog.rs`): a card over the window on a `@veil` backdrop,
up to 1040 × 720 and 24 px clear of the window edges, radius 14, hairline border.
- Left column (280 px; 200 px when the card is under 900 px wide): `SETTINGS`, a
  search field, then one entry per group (or per settings page): icon, bold title and
  a one-line summary in `dim_text` ("Theme, glow, motion"). The active entry is
  `@lift_active` with its icon in `accent`.
- Right column on `@lift`: a 64 px header with the group's title, an edit button for
  the config file and ✕, a hairline below; then the group's rows, its own heading hidden.
- Search filters every row across groups, shows each matching group with its heading,
  hides entries with no matches, and shows "No settings match “…”" when nothing does.
- <kbd>Esc</kbd> clears the search, then closes; a click on the backdrop closes.
- Other app-wide panels (the equalizer) use the same card with just a header and the page.

**Toolbar row** (`.toolbar`, `.list-toolbar`): 46 px, `0 12px 0 14px` padding, hairline
below. A toolbar of two rows (filters over a search) uses `.toolbar.stacked`: 10 px
padding and 10 px between the rows. A line of help above a list uses `.page-hint`
(12 px 14 px, hairline below, `dim_text`).

**Inputs:** transparent with a hairline border, radius 6, 32 px; the border turns
`accent` on focus, with no glow.

**Dropdown / popover:** 160 px minimum width; the closed dropdown ellipsizes. Popovers
use `surface`, a border, radius 10, 4 px padding, rows of radius 6 that hover to
`alpha(accent, 0.18)`.

**Tag:** a small label after a title for status that affects the whole row
("Restart needed", "Read-only"): 0.72em, bold, radius 4.

**Banner:** a full-width strip, radius 10, `alpha(accent, 0.10)` with a
`alpha(accent, 0.40)` border (warning: the same with `danger`), an icon, wrapped text
and an optional solid action button. Use it for status the user should act on.

**Toast:** bottom-centered over the window (above the status bar and any app bar), on
`surface`, radius 10; it replaces any toast showing and closes itself after 3.5 s
(6 s with an Undo button). Use toasts to confirm actions and report errors, never
modal alerts.

**Dialog:** a transient modal window (`.dialog-card`): `bg`, radius 14, 24 px padding,
a bold title, an optional mono code block on `alpha(shadow, 0.35)`, and buttons
right-aligned with the safe choice first ("Not now", then the action).

**Shortcuts list** (<kbd>F1</kbd>, also <kbd>?</kbd> where it doesn't clash): a dialog
with one card of rows, each an action and its key caps. Keep the list in one
`SHORTCUTS` constant that the settings Keyboard group reuses. Every app builds it the
same way (Fonts' `show_shortcuts` is the template):

- `widgets::dialog("Keyboard shortcuts", 520)`: the `section-title` heading, 520 px wide.
- One `.group-list` card of `widgets::row(action, "", key_caps)` rows. When the
  shortcuts are grouped (by context), each group gets an UPPERCASE `group-title`
  heading and its own card; the first heading has no margin above it.
- The list sits in a `ScrolledWindow` that scrolls without a scrollbar
  (`vscrollbar_policy(External)`, natural height up to 560 px), so no bar covers the keys.
- A plain **Close** button at the bottom right; <kbd>Esc</kbd> also closes. No other
  text in the dialog.

**Graphs** (Cairo on a `DrawingArea`): panel `mix(bg, surface, 0.5)`, radius 10, a
hairline border. Gridlines are `border` at 0.3–0.45 alpha, axis labels 11 px
`dim_text`, the line 2.5 px `accent` with a 0.14-alpha fill below it.
