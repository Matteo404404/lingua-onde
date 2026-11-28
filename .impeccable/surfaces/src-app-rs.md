---
version: 1
slug: "src-app-rs"
primary_target: "src/app.rs"
related_targets: ["src/theme.rs","src/session.rs"]
---

# App shell and home (Operate)

Scope: the whole desktop app shell (sidebar, top band, home, lesson session progress, buttons, cards), first surface the home screen. Mode: Operate. Audience: adult self-learners, 20–30 minutes a day (PRODUCT.md). Rename to Marea, visible only.

## Direction contract

THESIS: A course is a metro line and every lesson a station on it; refuses the category's rounded-card dashboard with progress rings and greetings.

OWN-WORLD: Milan-metro-era enamel signage. Warm grey ground, white enamel panels with a thick top band in the line colour, a near-black sidebar like a station wall. Each course language owns a line colour (its existing course colour); the line colour fills the band, the selected nav item and the primary button. Bold Noto Sans for signage titles, regular for reading. Square-ish 4–6 px corners. Level badges are ink squares with light letters.

STORY: The learner sees which line they are on, where today's station is, how far they have come and what is ahead, then presses one button to ride to today's lesson.

FIRST VIEWPORT: Left: 256 px dark rail with the Marea mark and wordmark, course switcher, nav (selected item filled in line colour). Top of content: 56 px band in line colour carrying the level badge and "Inglese · Fondamenti · direzione B2". Below: a white enamel panel with a 12 px line-colour top edge: "Prossima fermata · lezione 9 di 100", title 48–64 px bold, translation, primary button in line colour. Below it the signature: the line rail with stations (done filled, today large ring, ahead hollow, revisions as interchange double rings) and station names.

FORM: Pick card "The line" (Impeccable's pick, position 1 of 7 on my list), chosen by the user over the rolled record-sleeve direction. seed:9bdc0075a3a56a4e

Signature interaction: the line rail drawn wherever progress shows (home rail, the 10-step lesson progress as a 10-station line); hovering a station names it; the current station's ring eases in once.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
