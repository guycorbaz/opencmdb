//! The shell: the mock's frame, its ten addresses, and the nine screens that hold no data yet.
//!
//! # What this module is, and the one structural decision inside it
//!
//! Story 6b.2 gives the product more than one address for the first time. Ten screens, each
//! server-rendered at its own URL — no client-side router, no screen chosen by JavaScript.
//!
//! 🔴 **The nine demonstration screens live on a `Router<()>`, and that shape is a GUARD.**
//! Epic 6b's constraint 1 says no demo screen opens a database connection, and the validation
//! measured that forbidding it by discipline is worth nothing: on the main router — whose state
//! IS the pool — a handler may take `State<MySqlPool>` and it compiles cleanly. Merged *after*
//! `.with_state(pool)`, this sub-router's state is `()`, so the same handler **fails to
//! compile**. The shape is the carrier; the sentence is only its description.
//!
//! ⚠️ `/triage` is deliberately NOT here. It is the one screen `epics.md` feeds with the real
//! gap (story 6b.4), and it renders today's reconciliation card, so it needs the pool and stays
//! on the main router. Nine screens without, one with — constraint 1 is about demonstrations,
//! and it is satisfied exactly where it applies.

use axum::Router;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;

use crate::page::{Shell, render_shell};

/// One of the ten screens the navigation offers.
///
/// The order is the mock's, and so are the three groups. A screen is identified by the same key
/// the mock uses (`data-screen`), which keeps the two comparable when someone checks fidelity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    /// *Triage* — the loop's entry point, and the one screen fed by the real gap.
    Triage,
    /// *Tableau de bord* — the daily overview (story 6b.5).
    Dashboard,
    /// *Appareils* — the device list Epic 6's grouping will fill (6b.6).
    Devices,
    /// *Fiche appareil* — one device's record (6b.6).
    Device,
    /// *Applications* — Epic 15's frame (6b.7).
    Apps,
    /// *IPAM* — Epic 14's frame (6b.7).
    Ipam,
    /// *Sources* — where observations come from (6b.8).
    Sources,
    /// *Alertes* — Epic 16's frame (6b.8).
    Alerts,
    /// *Auto-diagnostic* — what the product knows about itself (6b.9).
    Diagnostic,
    /// *Mise en service* — first-run guidance (6b.9).
    Onboarding,
}

/// The three groups the mock's navigation is divided into, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NavGroup {
    /// *Boucle* — the daily loop.
    Loop,
    /// *Inventaire* — what is out there.
    Inventory,
    /// *Machine* — the product's own machinery.
    Machine,
}

impl Screen {
    /// Every screen, in the mock's navigation order.
    pub(crate) const ALL: [Screen; 10] = [
        Screen::Triage,
        Screen::Dashboard,
        Screen::Devices,
        Screen::Device,
        Screen::Apps,
        Screen::Ipam,
        Screen::Sources,
        Screen::Alerts,
        Screen::Diagnostic,
        Screen::Onboarding,
    ];

    /// The screen's own address — one URL per screen, deep-linkable cold.
    ///
    /// ⚠️ `/device` addresses no particular device, which is what the mock does and what Guy
    /// arbitrated for now. The honest shape is `/devices/{id}` and it needs an id that does not
    /// exist yet; the debt is registered with story 6b.6.
    pub(crate) fn href(self) -> &'static str {
        match self {
            Screen::Triage => "/triage",
            Screen::Dashboard => "/dashboard",
            Screen::Devices => "/devices",
            Screen::Device => "/device",
            Screen::Apps => "/apps",
            Screen::Ipam => "/ipam",
            Screen::Sources => "/sources",
            Screen::Alerts => "/alerts",
            Screen::Diagnostic => "/diagnostic",
            Screen::Onboarding => "/onboarding",
        }
    }

    /// The i18n key of the entry's label.
    pub(crate) fn label_key(self) -> &'static str {
        match self {
            Screen::Triage => "nav.triage",
            Screen::Dashboard => "nav.dashboard",
            Screen::Devices => "nav.devices",
            Screen::Device => "nav.device",
            Screen::Apps => "nav.apps",
            Screen::Ipam => "nav.ipam",
            Screen::Sources => "nav.sources",
            Screen::Alerts => "nav.alerts",
            Screen::Diagnostic => "nav.diagnostic",
            Screen::Onboarding => "nav.onboarding",
        }
    }

    /// Which of the mock's three groups the entry belongs to.
    pub(crate) fn group(self) -> NavGroup {
        match self {
            Screen::Triage | Screen::Dashboard => NavGroup::Loop,
            Screen::Devices | Screen::Device | Screen::Apps | Screen::Ipam => NavGroup::Inventory,
            Screen::Sources | Screen::Alerts | Screen::Diagnostic | Screen::Onboarding => {
                NavGroup::Machine
            }
        }
    }

    /// The i18n key of the screen's own title.
    pub(crate) fn title_key(self) -> &'static str {
        self.label_key()
    }
}

impl NavGroup {
    /// The groups in the mock's order.
    pub(crate) const ALL: [NavGroup; 3] = [NavGroup::Loop, NavGroup::Inventory, NavGroup::Machine];

    /// The i18n key of the group's heading.
    pub(crate) fn heading_key(self) -> &'static str {
        match self {
            NavGroup::Loop => "nav.group.loop",
            NavGroup::Inventory => "nav.group.inventory",
            NavGroup::Machine => "nav.group.machine",
        }
    }
}

/// The nine demonstration screens, on a **pool-free** router (see this module's doc).
///
/// # Returns
///
/// A `Router<()>` to be merged into the main router *after* its `.with_state(pool)`. The return
/// type is load-bearing: change it to `Router<MySqlPool>` and the whole guard evaporates.
pub(crate) fn router(perimeter: Option<String>) -> Router {
    let mut router = Router::new();
    for screen in Screen::ALL {
        if screen == Screen::Triage {
            continue; // fed by the real gap, and therefore on the main router
        }
        let perimeter = perimeter.clone();
        router = router.route(
            screen.href(),
            get(move || async move { empty_screen(screen, perimeter) }),
        );
    }
    router
}

/// A screen with no content yet: the frame, and nothing in the main column.
///
/// ⚠️ It renders empty ON PURPOSE. Story 6b.3 owns the example dataset and the marker that says
/// so; filling these screens here would satisfy this story's criteria while destroying the next
/// story's subject.
fn empty_screen(screen: Screen, perimeter: Option<String>) -> Response {
    Html(render_shell(Shell::new(screen, perimeter), String::new())).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::{Shell, render_shell};

    /// The rendered frame for one screen, with an empty body.
    fn shell_html(screen: Screen) -> String {
        render_shell(
            Shell::new(screen, Some("192.0.2.0/24".to_string())),
            String::new(),
        )
    }

    /// AC1 — ten entries, three groups, on every screen.
    #[test]
    fn every_screen_renders_ten_entries_in_three_groups() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            assert_eq!(
                html.matches("class=\"nav-entry\"").count(),
                10,
                "{screen:?}: the navigation offers ten screens from the first day — hiding one \
                 is story 6b.3's subject, not this story's escape hatch"
            );
            assert_eq!(
                html.matches("class=\"nav-group\"").count(),
                3,
                "{screen:?}: the mock's three groups (Boucle · Inventaire · Machine)"
            );
        }
    }

    /// AC1 — exactly one current entry. *At least one* would pass with all ten marked.
    #[test]
    fn exactly_one_entry_is_current_on_each_screen() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            assert_eq!(
                html.matches("aria-current=\"page\"").count(),
                1,
                "{screen:?}: exactly one entry is the current one"
            );
        }
    }

    /// AC1 — the version comes from the crate, never from a literal.
    ///
    /// A number typed into a template is a number that lies on a release day; the mock's own
    /// `v0.1.1 · maquette` is exactly such a literal, and the word *maquette* must not ship.
    #[test]
    fn the_version_is_the_crates_own_and_no_literal_is_typed() {
        let html = shell_html(Screen::Triage);
        assert!(
            html.contains(&format!("v{}", env!("CARGO_PKG_VERSION"))),
            "the header shows the crate's version"
        );
        assert!(!html.contains("maquette"), "the mock's word does not ship");
        let shell = include_str!("../templates/_shell.html");
        assert!(
            !shell.contains("v0."),
            "no version literal in the template — it must come from the crate"
        );
    }

    /// AC3 — every entry is OFFERED, not merely displayed, and the guard is a property.
    ///
    /// # What this guard does NOT see, stated rather than discovered
    ///
    /// The validation attacked an implementation of this ten ways and **two survived**:
    /// `display: none` written in `app.css` (which is where anyone would write it in a
    /// hand-authored-CSS story) and an inline `pointer-events: none`. Both leave markup this
    /// guard accepts. 🔑 **A guard covering them is not writable at the template-text level** —
    /// it needs computed styles, i.e. axe-core or a headless browser over the ten routes, which
    /// the epic's Definition of Done asks for and which no story yet owns (registered).
    #[test]
    fn every_entry_is_offered_with_its_own_address() {
        let html = shell_html(Screen::Triage);
        for screen in Screen::ALL {
            let anchor = format!("href=\"{}\"", screen.href());
            assert!(
                html.contains(&anchor),
                "{screen:?} must be reachable at its own address: {anchor} is absent"
            );
        }
        for withheld in [
            "href=\"#\"",
            "href=\"\"",
            "hidden",
            "aria-disabled",
            "display:none",
        ] {
            assert!(
                !html.contains(withheld),
                "an entry may not be displayed and withheld: found {withheld:?} — note the mock \
                 writes href=\"#\" on all ten, so transcribing it faithfully produces this"
            );
        }
    }

    /// AC2 — the screen is chosen by the ROUTE, not by JavaScript.
    #[test]
    fn no_screen_is_chosen_by_javascript() {
        let js = include_str!("../assets/app.js");
        for forbidden in ["data-screen", "pushState", "location.hash", "history."] {
            assert!(
                !js.contains(forbidden),
                "the mock switches screens client-side; this product does not: found {forbidden:?}"
            );
        }
    }

    /// AC2 — each screen renders its OWN identity, so a deep link lands where it says.
    #[test]
    fn each_screen_marks_itself_current() {
        for screen in Screen::ALL {
            let html = shell_html(screen);
            let marked = format!("href=\"{}\" aria-current=\"page\"", screen.href());
            assert!(
                html.contains(&marked),
                "{screen:?} must mark its OWN entry, or a deep link lands on the wrong screen"
            );
        }
    }

    /// AC6 — every key a screen references RESOLVES.
    ///
    /// 🔴 `rust-i18n` renders a missing key as its own NAME — no panic, not empty — so a typo
    /// ships as visible page text. The validation measured it: `nav.apps` → `nav.appz` put the
    /// literal `nav.appz` in the navigation with every other guard green.
    #[test]
    fn no_screen_renders_a_key_name_as_a_label() {
        let html = shell_html(Screen::Triage);
        for screen in Screen::ALL {
            assert!(
                !html.contains(screen.label_key()),
                "{:?} rendered its KEY instead of its label — rust-i18n falls back to the key \
                 name, so a typo is invisible to every other guard",
                screen
            );
        }
        for group in NavGroup::ALL {
            assert!(
                !html.contains(group.heading_key()),
                "{group:?} rendered its key instead of its heading"
            );
        }
    }

    /// AC6 — no key ever regresses to a single locale.
    ///
    /// 🔴 **This is the direction that is silent.** `rust-i18n` falls back to `en`, so deleting a
    /// FRENCH value renders English inside the French interface and no render-time guard can
    /// see it — measured: with `nav.alerts`' `fr` half removed, the whole suite stayed green.
    /// The English direction is loud by comparison. So the assertion is over the FILE, in both
    /// directions, rather than over a rendering.
    ///
    /// Baseline measured on `master`: 32 top-level entries, not one missing a locale. The guard
    /// is therefore *"no key regresses"*, never *"add the missing ones"*.
    #[test]
    fn every_key_carries_both_locales() {
        let yaml = include_str!("../locales/app.yml");
        let mut key = String::new();
        let mut locales: Vec<&str> = Vec::new();
        let mut checked = 0usize;
        let flush = |key: &str, locales: &mut Vec<&str>, checked: &mut usize| {
            if key.is_empty() || key == "_version" {
                locales.clear();
                return;
            }
            *checked += 1;
            assert!(
                locales.contains(&"en") && locales.contains(&"fr"),
                "{key} carries {locales:?} — a key with only `en` renders English inside the \
                 French UI, silently, because rust-i18n falls back"
            );
            locales.clear();
        };
        for line in yaml.lines() {
            let trimmed = line.trim_end();
            if trimmed.starts_with("  en:") {
                locales.push("en");
            } else if trimmed.starts_with("  fr:") {
                locales.push("fr");
            } else if let Some(name) = trimmed.strip_suffix(':')
                && !name.starts_with(' ')
                && !name.starts_with('#')
            {
                flush(&key, &mut locales, &mut checked);
                key = name.to_string();
            }
        }
        flush(&key, &mut locales, &mut checked);
        assert!(
            checked >= 47,
            "the premise: 48 entries minus `_version` ({checked} scanned) — a scan that found \
             nothing would assert nothing"
        );
    }

    /// AC1 — the perimeter comes from configuration, and its absence is said rather than blank.
    #[test]
    fn the_perimeter_is_shown_and_its_absence_is_named() {
        let configured = shell_html(Screen::Triage);
        assert!(
            configured.contains("192.0.2.0/24"),
            "the nav footer shows the perimeter the operator configured"
        );
        let unset = render_shell(Shell::new(Screen::Triage, None), String::new());
        assert!(
            unset.contains("not configured") || unset.contains("non configuré"),
            "an unset perimeter is NAMED, never rendered as an empty value"
        );
        assert!(
            !unset.contains("Perimeter </div>") && !unset.contains("Périmètre </div>"),
            "and never a dangling label"
        );
    }

    /// AC1 — the header carries no last observation, and no screen reads the clock or the store.
    ///
    /// ⚠️ Written over the WHOLE shell rather than over the `<header>` element: the validation
    /// measured that a guard scoped to the header passes over a shell carrying both facts in the
    /// sidebar — *the guard placed where the defect cannot occur*.
    #[test]
    fn the_shell_shows_no_last_observation() {
        // 🔑 Comments stripped first — the same idiom `float-free` uses so the architecture may
        // be QUOTED in a comment without tripping a gate, and the same trap story 6b.1 met when
        // its radius comment contained the very text its scanner counted. This template's own
        // comment names the deferred fact in order to explain it.
        let source = include_str!("../templates/_nav.html");
        let code: String = source
            .split("{#")
            .enumerate()
            .map(|(i, part)| {
                if i == 0 {
                    part.to_string()
                } else {
                    part.split_once("#}")
                        .map(|(_, rest)| rest)
                        .unwrap_or("")
                        .to_string()
                }
            })
            .collect();
        assert!(
            !code.to_lowercase().contains("observ"),
            "the last observation is a MAX(observed_at) — a database read on ten screens, eight \
             of them demonstrations. Deferred to story 6b.5 (registered), and the ABSENCE is \
             asserted because it is a decision, not an omission"
        );
    }
}
