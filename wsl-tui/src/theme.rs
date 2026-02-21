//! Catppuccin Mocha colour constants for WSL TUI.
//!
//! All constants are [`ratatui::style::Color::Rgb`] values sourced from the
//! official Catppuccin Mocha palette and verified against
//! `docs/THEME_GUIDELINES.md`.
//!
//! # Usage
//!
//! Import individual constants where needed:
//!
//! ```rust
//! use wsl_tui::theme::{MAUVE, BASE, TEXT};
//! ```
//!
//! Or import the whole module:
//!
//! ```rust
//! use wsl_tui::theme;
//! // theme::MAUVE, theme::BASE, etc.
//! ```

use ratatui::style::Color;

// ── Accent colors (functional) ───────────────────────────────────────────────

/// Primary accent — selected items, active tab, focus ring, primary buttons.
///
/// Catppuccin Mocha "Mauve" `#cba6f7`.
pub const MAUVE: Color = Color::Rgb(203, 166, 247);

/// Secondary accent — links, secondary actions, informational highlights.
///
/// Catppuccin Mocha "Blue" `#89b4fa`.
pub const BLUE: Color = Color::Rgb(137, 180, 250);

/// Running state, success messages, applied packs, gauges (OK).
///
/// Catppuccin Mocha "Green" `#a6e3a1`.
pub const GREEN: Color = Color::Rgb(166, 227, 161);

/// Warnings, pending state, attention indicators.
///
/// Catppuccin Mocha "Yellow" `#f9e2af`.
pub const YELLOW: Color = Color::Rgb(249, 226, 175);

/// Errors, stopped state, failed steps, destructive actions.
///
/// Catppuccin Mocha "Red" `#f38ba8`.
pub const RED: Color = Color::Rgb(243, 139, 168);

/// Tips, help text, informational banners.
///
/// Catppuccin Mocha "Sapphire" `#74c7ec`.
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);

/// Search matches, highlighted text, special indicators.
///
/// Catppuccin Mocha "Peach" `#fab387`.
pub const PEACH: Color = Color::Rgb(250, 179, 135);

/// Tab headers, section titles, category labels.
///
/// Catppuccin Mocha "Lavender" `#b4befe`.
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);

/// Connection status, terminal indicators, Termius.
///
/// Catppuccin Mocha "Teal" `#94e2d5`.
pub const TEAL: Color = Color::Rgb(148, 226, 213);

/// Pack categories, wizard step indicators, provisioning progress.
///
/// Catppuccin Mocha "Pink" `#f5c2e7`.
pub const PINK: Color = Color::Rgb(245, 194, 231);

/// CPU usage gauge and charts.
///
/// Catppuccin Mocha "Flamingo" `#f2cdcd`.
pub const FLAMINGO: Color = Color::Rgb(242, 205, 205);

/// Memory usage gauge and charts.
///
/// Catppuccin Mocha "Rosewater" `#f5e0dc`.
pub const ROSEWATER: Color = Color::Rgb(245, 224, 220);

// ── Surface and text colors ───────────────────────────────────────────────────

/// Primary text, headings.
///
/// Catppuccin Mocha "Text" `#cdd6f4`.
pub const TEXT: Color = Color::Rgb(205, 214, 244);

/// Secondary text, descriptions.
///
/// Catppuccin Mocha "Subtext1" `#bac2de`.
pub const SUBTEXT1: Color = Color::Rgb(186, 194, 222);

/// Placeholder text, disabled items, muted content.
///
/// Catppuccin Mocha "Subtext0" `#a6adc8`.
pub const SUBTEXT0: Color = Color::Rgb(166, 173, 200);

/// Comments, timestamps, metadata — dimmed text.
///
/// Catppuccin Mocha "Overlay0" `#6c7086`.
pub const OVERLAY0: Color = Color::Rgb(108, 112, 134);

/// Scrollbars, subtle UI elements.
///
/// Catppuccin Mocha "Surface2" `#585b70`.
pub const SURFACE2: Color = Color::Rgb(88, 91, 112);

/// Borders, separators, inactive tabs.
///
/// Catppuccin Mocha "Surface1" `#45475a`.
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);

/// Panels, cards, raised surfaces.
///
/// Catppuccin Mocha "Surface0" `#313244`.
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);

/// Main background.
///
/// Catppuccin Mocha "Base" `#1e1e2e`.
pub const BASE: Color = Color::Rgb(30, 30, 46);

/// Status bar background.
///
/// Catppuccin Mocha "Mantle" `#181825`.
pub const MANTLE: Color = Color::Rgb(24, 24, 37);

/// Deepest background — outermost frame or overlay backdrop.
///
/// Catppuccin Mocha "Crust" `#11111b`.
pub const CRUST: Color = Color::Rgb(17, 17, 27);

#[cfg(test)]
mod tests {
    use super::*;

    /// All constants must be `Color::Rgb` variants — never named colours.
    #[test]
    fn test_theme_colors_are_rgb() {
        // Accent colors
        assert!(matches!(MAUVE, Color::Rgb(_, _, _)));
        assert!(matches!(BLUE, Color::Rgb(_, _, _)));
        assert!(matches!(GREEN, Color::Rgb(_, _, _)));
        assert!(matches!(YELLOW, Color::Rgb(_, _, _)));
        assert!(matches!(RED, Color::Rgb(_, _, _)));
        assert!(matches!(SAPPHIRE, Color::Rgb(_, _, _)));
        assert!(matches!(PEACH, Color::Rgb(_, _, _)));
        assert!(matches!(LAVENDER, Color::Rgb(_, _, _)));
        assert!(matches!(TEAL, Color::Rgb(_, _, _)));
        assert!(matches!(PINK, Color::Rgb(_, _, _)));
        assert!(matches!(FLAMINGO, Color::Rgb(_, _, _)));
        assert!(matches!(ROSEWATER, Color::Rgb(_, _, _)));

        // Surface and text colors
        assert!(matches!(TEXT, Color::Rgb(_, _, _)));
        assert!(matches!(SUBTEXT1, Color::Rgb(_, _, _)));
        assert!(matches!(SUBTEXT0, Color::Rgb(_, _, _)));
        assert!(matches!(OVERLAY0, Color::Rgb(_, _, _)));
        assert!(matches!(SURFACE2, Color::Rgb(_, _, _)));
        assert!(matches!(SURFACE1, Color::Rgb(_, _, _)));
        assert!(matches!(SURFACE0, Color::Rgb(_, _, _)));
        assert!(matches!(BASE, Color::Rgb(_, _, _)));
        assert!(matches!(MANTLE, Color::Rgb(_, _, _)));
        assert!(matches!(CRUST, Color::Rgb(_, _, _)));
    }

    /// BASE must have low RGB values to confirm this is the dark Mocha variant,
    /// not the light Latte variant.
    #[test]
    fn test_base_is_dark() {
        if let Color::Rgb(r, g, b) = BASE {
            // Catppuccin Mocha Base is #1e1e2e (30, 30, 46) — well below 128.
            assert!(r < 128, "BASE red channel too high for Mocha: {r}");
            assert!(g < 128, "BASE green channel too high for Mocha: {g}");
            assert!(b < 128, "BASE blue channel too high for Mocha: {b}");
        } else {
            panic!("BASE is not Color::Rgb");
        }
    }

    /// Spot-check exact RGB values against THEME_GUIDELINES.md hex table.
    #[test]
    fn test_palette_values_match_spec() {
        // Mauve #cba6f7 → (203, 166, 247)
        assert_eq!(MAUVE, Color::Rgb(203, 166, 247));
        // Green #a6e3a1 → (166, 227, 161)
        assert_eq!(GREEN, Color::Rgb(166, 227, 161));
        // Red #f38ba8 → (243, 139, 168)
        assert_eq!(RED, Color::Rgb(243, 139, 168));
        // Base #1e1e2e → (30, 30, 46)
        assert_eq!(BASE, Color::Rgb(30, 30, 46));
        // Mantle #181825 → (24, 24, 37)
        assert_eq!(MANTLE, Color::Rgb(24, 24, 37));
        // Crust #11111b → (17, 17, 27)
        assert_eq!(CRUST, Color::Rgb(17, 17, 27));
        // Text #cdd6f4 → (205, 214, 244)
        assert_eq!(TEXT, Color::Rgb(205, 214, 244));
        // Surface0 #313244 → (49, 50, 68)
        assert_eq!(SURFACE0, Color::Rgb(49, 50, 68));
        // Surface1 #45475a → (69, 71, 90)
        assert_eq!(SURFACE1, Color::Rgb(69, 71, 90));
        // Sapphire #74c7ec → (116, 199, 236)
        assert_eq!(SAPPHIRE, Color::Rgb(116, 199, 236));
        // Teal #94e2d5 → (148, 226, 213)
        assert_eq!(TEAL, Color::Rgb(148, 226, 213));
        // Lavender #b4befe → (180, 190, 254)
        assert_eq!(LAVENDER, Color::Rgb(180, 190, 254));
    }
}
