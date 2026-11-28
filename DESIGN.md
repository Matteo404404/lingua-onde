---
name: Marea
description: A slow daily language method drawn as a metro line, in enamel station signage.
colors:
  line-en: "#284A9A"
  line-es: "#C43222"
  line-fr: "#2C58B2"
  line-de: "#3A3A44"
  line-nl: "#B84E0C"
  line-pt: "#1B7D44"
  line-zh: "#B4222E"
  line-la: "#8A622E"
  line-vi: "#E0A81E"
  line-it: "#208048"
  brand-enamel-red: "#C83424"
  station-ground: "#F3F2EF"
  station-ground-dark: "#121212"
  enamel-panel: "#FFFFFF"
  enamel-panel-dark: "#1E1E1E"
  sign-ink: "#1C1C1C"
  sign-ink-dark: "#ECECEC"
  muted: "#646464"
  muted-dark: "#9E9E9E"
  hairline: "#E2DFDA"
  hairline-dark: "#343434"
  track: "#E8E5E0"
  track-dark: "#323232"
  station-wall: "#1C1C1C"
  station-wall-dark: "#0C0C0C"
  wall-text: "#D6D6D6"
  wall-muted: "#969696"
  field: "#F7F6F3"
  field-dark: "#181818"
  control: "#EAE7E2"
  control-dark: "#383838"
  control-line: "#D2CEC7"
  control-line-dark: "#4C4C4C"
  pending: "#C4C0B8"
  pending-dark: "#5A5A5A"
  good: "#208048"
  good-dark: "#54BE7A"
  good-soft: "#E0F0E5"
  good-soft-dark: "#1E3426"
  warn: "#A06008"
  warn-dark: "#E6A646"
  warn-soft: "#FAEED6"
  warn-soft-dark: "#3E301A"
  bad: "#C03C32"
  bad-dark: "#EC6C62"
  highlight: "#FFF0C4"
  highlight-dark: "#4A3E1C"
  figured: "#965C1E"
  figured-dark: "#DEA868"
typography:
  display:
    fontFamily: "Archivo, Noto Sans, sans-serif"
    fontSize: "52px"
    fontWeight: 700
  headline:
    fontFamily: "Archivo, Noto Sans, sans-serif"
    fontSize: "30px"
    fontWeight: 700
  title:
    fontFamily: "Archivo, Noto Sans, sans-serif"
    fontSize: "18px"
    fontWeight: 700
  sign:
    fontFamily: "Archivo, Noto Sans, sans-serif"
    fontSize: "17px"
    fontWeight: 600
  body:
    fontFamily: "Noto Sans, sans-serif"
    fontSize: "15.5px"
    fontWeight: 400
  button:
    fontFamily: "Noto Sans, sans-serif"
    fontSize: "15px"
    fontWeight: 500
  label:
    fontFamily: "Noto Sans, sans-serif"
    fontSize: "13px"
    fontWeight: 500
  small:
    fontFamily: "Noto Sans, sans-serif"
    fontSize: "12.5px"
    fontWeight: 400
rounded:
  tile: "3px"
  control: "4px"
  panel: "6px"
spacing:
  item: "8px"
  tinted-pad: "16px"
  card-gap: "16px"
  card-pad: "20px"
  gutter: "22px"
components:
  button-primary:
    backgroundColor: "{colors.line-en}"
    textColor: "{colors.enamel-panel}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "7px 14px"
    height: "36px"
  button-primary-hover:
    backgroundColor: "#234188"
  button-primary-active:
    backgroundColor: "#203B7B"
  button-primary-big:
    backgroundColor: "{colors.line-en}"
    textColor: "{colors.enamel-panel}"
    rounded: "{rounded.control}"
    height: "48px"
    width: "200px"
  button-secondary:
    backgroundColor: "{colors.enamel-panel}"
    textColor: "{colors.sign-ink}"
    typography: "{typography.button}"
    rounded: "{rounded.control}"
    padding: "7px 14px"
    height: "36px"
  button-secondary-hover:
    backgroundColor: "#F4F6FA"
  button-ghost:
    textColor: "{colors.muted}"
    rounded: "{rounded.control}"
    padding: "7px 14px"
  card:
    backgroundColor: "{colors.enamel-panel}"
    rounded: "{rounded.panel}"
    padding: "20px"
  line-band:
    backgroundColor: "{colors.line-en}"
    textColor: "{colors.enamel-panel}"
    typography: "{typography.sign}"
    rounded: "{rounded.control}"
    height: "52px"
  level-badge:
    backgroundColor: "{colors.sign-ink}"
    textColor: "{colors.station-ground}"
    rounded: "{rounded.tile}"
    padding: "1px 6px"
  nav-item:
    textColor: "{colors.wall-text}"
    rounded: "{rounded.control}"
    height: "36px"
  nav-item-selected:
    backgroundColor: "{colors.line-en}"
    textColor: "{colors.enamel-panel}"
  segmented-track:
    backgroundColor: "{colors.track}"
    rounded: "{rounded.panel}"
    padding: "4px"
  segmented-chip:
    backgroundColor: "{colors.enamel-panel}"
    textColor: "{colors.sign-ink}"
    rounded: "{rounded.control}"
    height: "34px"
  text-field:
    backgroundColor: "{colors.field}"
    textColor: "{colors.sign-ink}"
    rounded: "{rounded.control}"
---

# Design System: Marea

## Overview

**Creative North Star: "The Station Sign"**

Marea treats a course as a metro line and every lesson as a station on it. The whole interface is lettered like mid-century enamel station signage: a warm grey ground, white enamel panels with a thin hairline, a near-black wall for the sidebar, and one line colour per course that does all the colour work. The chrome itself is colourless; colour arrives only from the line the learner is riding.

Density is calm and book-like. One centred column of panels, generous padding inside each, big Archivo titles over quiet Noto Sans reading text. Depth is flat: panels separate from the ground by tone and a 1px hairline, never by shadow. Motion is small and purposeful: the current station's ring grows in once, a pointed station swells a little, the segmented chip and the lesson stepper glide with a strong ease-out.

The signature is the line rail, drawn wherever progress shows: the home path, the ten-step lesson stepper. Done stations are filled, today's is a large ring, those ahead are hollow, revisions carry a second interchange ring.

**Key Characteristics:**
- One line colour per course, set each frame; it fills the primary button, the selected nav item, the band and the travelled rail.
- Archivo Bold and SemiBold for signage (titles, band, badges, station names); Noto Sans for everything read.
- Square-ish corners: 3px tiles, 4px controls, 6px panels.
- Flat enamel panels on a warm grey ground; a near-black station wall on the left.
- Drawn one-stroke icons, never emoji or typed symbols.
- Every colour has a light and a dark pair.

## Colors

A colourless enamel-and-ink chrome; the course's line colour is the only accent, with semantic greens, ambers and reds kept for status.

### Primary
- **The Line Colour** (`line-*`, default `line-en`): each course language owns one (English deep cobalt, Spanish signal red, French clear blue, German slate, Dutch burnt orange, Portuguese and Italian greens, Chinese crimson, Latin bronze, Vietnamese saffron). It is the fill of the primary button, the selected nav item, the 52px band above home, the 12px enamel header strip of the hero panel, the language tile, and the travelled part of the rail. Derived steps come from it at runtime, never as fixed tokens: *accent* (line as ink for links and focus rings; darkened 45% toward black when the line is light, lifted 35% toward white on dark), *accent-soft* (line mixed 88% into the panel, 78% on dark: selected options, tinted notice panels), *hover* (95% / 88%), *hover-fill* (80% / 68%), *accent-dark* (25% toward black: pressed control stroke).
- **Text on the line** is white, or sign ink when the line's WCAG luminance exceeds 0.3 (Vietnamese saffron).

### Secondary
- **Brand Enamel Red** (`brand-enamel-red`): the Marea logo tile only. It is not a line and never fills a control.

### Neutral
- **Station Ground** (`station-ground`): the app background behind every panel.
- **Enamel Panel** (`enamel-panel`): cards, windows, the selected segmented chip, the inside of hollow stations.
- **Sign Ink** (`sign-ink`): all text by default; also the fill of level badges and toasts.
- **Muted** (`muted`): subtitles, translations, ghost buttons, unselected segments, station numbers.
- **Hairline** (`hairline`): the 1px stroke around panels and windows.
- **Track** (`track`): progress-bar tracks, segmented-control tracks, inactive number circles.
- **Station Wall** (`station-wall`) with **Wall Text** and **Wall Muted**: the left navigation panel and its labels and counts.
- **Field** (`field`): text-box backgrounds, a step off the panel so inputs read on cards.
- **Control / Control Line** (`control`, `control-line`): checkboxes, slider rails, secondary button borders, keycaps.
- **Pending** (`pending`): lessons not yet started.

### Status
- **Good / Warn / Bad** with **soft** fills: answers right and wrong, update notices, the review count dot. Good and Warn are dark enough on light to be read as text (4.5:1).
- **Highlight** (`highlight`): the tinted panel behind a sentence being worked on.
- **Figured** (`figured`): the spelled-out pronunciation line, a warm brown that reads as a printed annotation.

### Named Rules
**The One Line Rule.** Colour on the chrome comes from the course's line colour and nothing else. A new surface takes `FILL`/`ACCENT` and their mixes; it never introduces its own accent hue.

**The Dark Pair Rule.** Every palette entry is declared as a light and a dark pair in `theme.rs`; marks drawn on the ground in a line colour go through `mark()` so they are lifted on dark.

## Typography

**Display Font:** Archivo Bold (falls back to Noto Sans, then the CJK system font)
**Sign Font:** Archivo SemiBold
**Body Font:** Noto Sans Regular, with Medium for buttons and labels and Bold for stressed syllables

**Character:** Archivo is the station sign's lettering, compact and confident; Noto Sans is the reading voice, chosen because it covers Vietnamese tone letters and every Latin, Greek and Cyrillic accent.

### Hierarchy
- **Display** (Archivo 700, 52px): the hero title of today's lesson.
- **Headline** (Archivo 700, 30px): page titles and the finished/empty states of the hero; the egui heading style is 26px.
- **Title** (Archivo 700, 18–19px): card and settings section titles ("Your path", "Appearance").
- **Sign** (Archivo 600, 17px band; 14px band tile, language tile and today's station name; 12.5px level badges): signage text on a line or ink fill.
- **Body** (Noto Sans 400, 15.5px): reading text; translations under the hero title at 19px muted.
- **Button** (Noto Sans 500, 15px; 16px on the big primary): every button and nav label.
- **Label** (Noto Sans 500, 13px, muted, sentence case): small labels above a group or value.
- **Small** (Noto Sans 400, 12.5px): station numbers, metadata, counts.

### Named Rules
**The Signage Voice Rule.** Archivo is for things a station sign would say: titles, the band, badges, station names. Anything the learner reads at length is Noto Sans.

**The Sentence Case Rule.** Labels are sentence case at 13px medium muted. No uppercase tracking.

## Layout

A fixed left station wall (232px; 72px icon-only below 820px of window width) and a scrolling central column. Content is centred at a maximum page width with a 22px gutter each side, never narrower than 320px. Home stacks full-width panels with 16px between them: the 52px line band, the hero panel, the path card with its rail, the study calendar, the languages card. Cards pad 20px; the hero pads 28px sides, 36px top (room for its 12px enamel header), 24px bottom; tinted notice panels pad 16px. Base item spacing is 8px; button padding 14×7px; interactive height 32px, buttons 36px. The lesson session replaces the page with a top bar, the 10-station stepper, the step's content and a bottom bar of Back / Next.

## Elevation & Depth

Flat. Panels sit on the ground by tone (white enamel on warm grey) and a 1px hairline. Shadows appear only on things that float above the page: windows (`0 2px 18px rgba(0,0,0,0.094)`), popups and menus (`0 2px 12px rgba(0,0,0,0.078)`), and the toast (`0 4px 16px rgba(0,0,0,0.196)`). Pressed controls sink by one pixel instead of lifting.

### Named Rules
**The Enamel Flat Rule.** Nothing on the page casts a shadow. Only floating layers (windows, popups, toasts) do.

## Shapes

Square-ish, signage geometry. Tiles and badges 3px; buttons, fields, checkboxes, nav items, the band and segmented chips 4px; cards, windows, menus, the segmented track and the toast 6px. The hero's enamel header strip rounds only its top corners (6px) to sit on the card. Language tiles round at 12% of their side. Circles are reserved for stations, number markers, round play buttons and icon circles. The logo is a red enamel tile (radius ≈22% of the side) carrying a white station ring half filled by a wave.

## Components

### Buttons
One button system (`Btn`), every variant answering hover and press.
- **Shape:** gently squared (4px), 36px high; the big primary is 200×48px at 16px.
- **Primary:** line-colour fill, text on the line; hover darkens 12% toward black, press 20%; no stroke.
- **Secondary:** enamel panel with a 1px control-line stroke and ink text; hover takes the hover step of the line and an accent stroke; press takes accent-soft with an accent-dark stroke and sinks 1px.
- **Ghost:** no frame, muted text; the hover surface shows without a stroke.
- **On-line:** a quiet action on a line band, in the band's text colour; hover and press lay that colour at 14% and 22% alpha.
- **Text link:** muted underlined text that gains a 4px hover-surface behind it.
- **Icon buttons:** a 28px square ghost with a 13px drawn icon (12px pressed), muted turning ink on hover; round filled buttons whose radius shrinks to 47% when pressed. Focus ring: 1.5px accent outside.

### Chips / Badges
- **Level badge:** sign ink square, ground-coloured Archivo SemiBold 12.5px letters, 3px corners. Every level is a stretch of one line, so badges are never coloured per level.
- **Band tile:** on the band the level sits on a 34×26px tile of the band's text colour with letters in the line colour.
- **Language tile:** the code in Archivo SemiBold on a square of the language's line colour.
- **Pill:** a status colour on its soft fill, 4px, 13px medium.

### Cards / Containers
- **Corner Style:** 6px.
- **Background:** enamel panel; tinted panels use accent-soft, highlight or warn-soft with no stroke.
- **Shadow Strategy:** none (see Elevation).
- **Border:** 1px hairline.
- **Internal Padding:** 20px; tinted 16px.

### Inputs / Fields
- **Style:** field background a step off the panel, 4px corners, 1px control-line stroke.
- **Focus:** 1.5px accent stroke (the selection stroke doubles as the focus ring).
- **Selected option:** accent-soft fill with ink text.

### Segmented Control
A track-coloured 6px tray padded 4px; options are 34px transparent buttons, muted text; the chosen one sits on a white 4px chip that glides to a new choice over 0.18s ease-out; hover on an unselected option shows a half step between track and panel.

### Navigation
The station wall: near-black, padded 12×18px, Marea mark (32px) and Archivo wordmark at 24px in white, a course switcher with the language tile, then 36px nav items with a 16px drawn icon and a 15px medium label. Selected: line-colour fill, text on the line. Hover: white at ~5% alpha. Review counts sit right in wall-muted; in compact mode a warn dot replaces the count.

### The Line Rail (signature)
A thick line (22% of its height, 4–10px) in line ink up to today's station, then the lighter line track. Stations: done = filled line circle with a panel-coloured centre; today = a large ring (1.5× radius, 0.55× stroke) that grows in once over 0.24s; ahead = hollow panel circle with a track stroke; a revision adds a 1.5px interchange ring at 1.9× radius. A pointed station grows 15% over 0.12s. Labels under the stations: today's in Archivo SemiBold 14px ink, others Noto Sans 12.5px muted, collisions hidden rather than overlapped. The lesson stepper is the same line at 5px with 10 stations, its travelled end gliding over 0.22s.

### Icons
One drawn set (home, play, pause, stop, review, lines, book, pencil, sliders, check, speaker, close, back, forward, arrow, station, dot, rewind, skip, up), painted with a stroke of 10% of their size (1.5–3px), taking the colour of the text beside them.

## Do's and Don'ts

### Do:
- **Do** take every accent from the course's line colour (`FILL`, `ACCENT`, `ACCENT_SOFT`, `HOVER`) and put text on it with `text_on`.
- **Do** declare any new colour as a light and dark pair in `theme.rs`.
- **Do** set titles and signage in Archivo (`display`, `sign_family`) and reading text in Noto Sans.
- **Do** use the `Btn` variants (primary, secondary, ghost, on-line) so every button answers hover and press.
- **Do** draw progress as the line rail: filled done, ringed today, hollow ahead, double-ringed revisions.
- **Do** keep corners at 3px, 4px or 6px and panels flat with a 1px hairline.
- **Do** ease marks in with the shared cubic ease-out (`ease`, `glide`) at 0.12–0.24s.

### Don't:
- **Don't** give the chrome its own accent hue or use the brand red outside the logo.
- **Don't** colour level badges per level; they are ink squares on one line.
- **Don't** put shadows under panels on the page.
- **Don't** use emoji or typed symbols as icons; draw them with `Icon`.
- **Don't** set uppercase tracked labels; labels are sentence case.
- **Don't** use Archivo for paragraphs, translations or notes.
