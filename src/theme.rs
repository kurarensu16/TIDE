//! Color Themes for TIDE workspace.
//!
//! Provides curated palettes (Tokyo Night, Catppuccin Mocha, Nord, Gruvbox, Cyberpunk, and TIDE Ocean)
//! with live preview modal, hotkey switching, and centralized palette styling tokens.

use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemePalette {
    pub bg: Color,
    pub surface: Color,
    pub surface_variant: Color,
    pub border_active: Color,
    pub border_inactive: Color,
    pub accent: Color,
    pub accent_secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,
    pub tab_inactive_bg: Color,
    pub tab_inactive_fg: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ColorTheme {
    #[default]
    TideOcean,
    TokyoNight,
    CatppuccinMocha,
    Nord,
    GruvboxDark,
    Cyberpunk,
    Dracula,
    RosePine,
    MonokaiPro,
    OneDarkPro,
}

impl ColorTheme {
    pub const ALL: [ColorTheme; 10] = [
        ColorTheme::TideOcean,
        ColorTheme::TokyoNight,
        ColorTheme::CatppuccinMocha,
        ColorTheme::Nord,
        ColorTheme::GruvboxDark,
        ColorTheme::Cyberpunk,
        ColorTheme::Dracula,
        ColorTheme::RosePine,
        ColorTheme::MonokaiPro,
        ColorTheme::OneDarkPro,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            ColorTheme::TideOcean => "TIDE Ocean (Default)",
            ColorTheme::TokyoNight => "Tokyo Night",
            ColorTheme::CatppuccinMocha => "Catppuccin Mocha",
            ColorTheme::Nord => "Nord Arctic",
            ColorTheme::GruvboxDark => "Gruvbox Dark",
            ColorTheme::Cyberpunk => "Cyberpunk Neon",
            ColorTheme::Dracula => "Dracula",
            ColorTheme::RosePine => "Rosé Pine",
            ColorTheme::MonokaiPro => "Monokai Pro",
            ColorTheme::OneDarkPro => "One Dark Pro",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ColorTheme::TideOcean => "Deep oceanic slate blue with vibrant cyan and electric wave accents",
            ColorTheme::TokyoNight => "Clean midnight blue storm aesthetic with soft neon lavender & sky blue",
            ColorTheme::CatppuccinMocha => "Soothing, luxurious dark pastel palette with mauve, lavender, and peach",
            ColorTheme::Nord => "Arctic, north-bluish cold aesthetic with frost blues and polar snow tones",
            ColorTheme::GruvboxDark => "Warm retro groove palette with golden amber, forest aqua, and burnt orange",
            ColorTheme::Cyberpunk => "High-voltage synthwave with neon fuchsia pink, electric cyan, and laser yellow",
            ColorTheme::Dracula => "Gothic dark purple-charcoal with neon orchid pink, cyan, and lime accents",
            ColorTheme::RosePine => "Alluring dark pine dusk palette with blush rose, lilac iris, and foam gold",
            ColorTheme::MonokaiPro => "Volcanic matte charcoal with sublime yellow, coral magenta, and mint green",
            ColorTheme::OneDarkPro => "Clean neutral graphite slate with iconic chalk blue, purple, and sage green",
        }
    }

    /// Preview color swatches: (Background, Accent 1, Accent 2, Success)
    pub fn swatches(&self) -> (Color, Color, Color, Color) {
        let p = self.palette();
        (p.surface, p.accent, p.accent_secondary, p.success)
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            ColorTheme::TideOcean => ColorTheme::TokyoNight,
            ColorTheme::TokyoNight => ColorTheme::CatppuccinMocha,
            ColorTheme::CatppuccinMocha => ColorTheme::Nord,
            ColorTheme::Nord => ColorTheme::GruvboxDark,
            ColorTheme::GruvboxDark => ColorTheme::Cyberpunk,
            ColorTheme::Cyberpunk => ColorTheme::Dracula,
            ColorTheme::Dracula => ColorTheme::RosePine,
            ColorTheme::RosePine => ColorTheme::MonokaiPro,
            ColorTheme::MonokaiPro => ColorTheme::OneDarkPro,
            ColorTheme::OneDarkPro => ColorTheme::TideOcean,
        }
    }

    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        match self {
            ColorTheme::TideOcean => ColorTheme::OneDarkPro,
            ColorTheme::TokyoNight => ColorTheme::TideOcean,
            ColorTheme::CatppuccinMocha => ColorTheme::TokyoNight,
            ColorTheme::Nord => ColorTheme::CatppuccinMocha,
            ColorTheme::GruvboxDark => ColorTheme::Nord,
            ColorTheme::Cyberpunk => ColorTheme::GruvboxDark,
            ColorTheme::Dracula => ColorTheme::Cyberpunk,
            ColorTheme::RosePine => ColorTheme::Dracula,
            ColorTheme::MonokaiPro => ColorTheme::RosePine,
            ColorTheme::OneDarkPro => ColorTheme::MonokaiPro,
        }
    }

    pub fn palette(&self) -> ThemePalette {
        match self {
            ColorTheme::TideOcean => ThemePalette {
                bg: Color::Rgb(11, 15, 25),
                surface: Color::Rgb(15, 23, 42),
                surface_variant: Color::Rgb(30, 41, 59),
                border_active: Color::Rgb(56, 189, 248), // Electric cyan
                border_inactive: Color::Rgb(30, 41, 59), // Slate 800
                accent: Color::Rgb(56, 189, 248),        // Cyan 400
                accent_secondary: Color::Rgb(168, 85, 247), // Purple 500
                success: Color::Rgb(34, 197, 94),        // Emerald 500
                warning: Color::Rgb(234, 179, 8),        // Amber 500
                danger: Color::Rgb(239, 68, 68),         // Red 500
                text_primary: Color::Rgb(248, 250, 252),  // Slate 50
                text_secondary: Color::Rgb(203, 213, 225),// Slate 300
                text_muted: Color::Rgb(100, 116, 139),    // Slate 500
                tab_active_bg: Color::Rgb(30, 58, 138),  // Blue 900
                tab_active_fg: Color::White,
                tab_inactive_bg: Color::Rgb(15, 23, 42), // Slate 900
                tab_inactive_fg: Color::Rgb(148, 163, 184),
                selection_bg: Color::Rgb(30, 58, 138),
                selection_fg: Color::White,
            },
            ColorTheme::TokyoNight => ThemePalette {
                bg: Color::Rgb(22, 22, 30),
                surface: Color::Rgb(26, 27, 38),
                surface_variant: Color::Rgb(36, 40, 59),
                border_active: Color::Rgb(122, 162, 247), // Tokyo blue
                border_inactive: Color::Rgb(41, 46, 66),  // Tokyo border
                accent: Color::Rgb(122, 162, 247),        // Sky blue
                accent_secondary: Color::Rgb(187, 154, 247), // Lavender magenta
                success: Color::Rgb(158, 206, 106),       // Tokyo green
                warning: Color::Rgb(224, 175, 104),       // Tokyo yellow/orange
                danger: Color::Rgb(247, 118, 142),        // Tokyo red
                text_primary: Color::Rgb(192, 202, 245),  // Foreground
                text_secondary: Color::Rgb(169, 177, 214),
                text_muted: Color::Rgb(86, 95, 137),      // Comment
                tab_active_bg: Color::Rgb(41, 46, 66),
                tab_active_fg: Color::Rgb(122, 162, 247),
                tab_inactive_bg: Color::Rgb(26, 27, 38),
                tab_inactive_fg: Color::Rgb(86, 95, 137),
                selection_bg: Color::Rgb(40, 52, 87),
                selection_fg: Color::White,
            },
            ColorTheme::CatppuccinMocha => ThemePalette {
                bg: Color::Rgb(17, 17, 27),               // Crust
                surface: Color::Rgb(30, 30, 46),          // Base
                surface_variant: Color::Rgb(49, 50, 68),  // Surface0
                border_active: Color::Rgb(203, 166, 247), // Mauve
                border_inactive: Color::Rgb(69, 71, 90),  // Surface1
                accent: Color::Rgb(180, 190, 254),        // Lavender
                accent_secondary: Color::Rgb(116, 199, 236), // Sapphire
                success: Color::Rgb(166, 227, 161),       // Green
                warning: Color::Rgb(249, 226, 175),       // Yellow
                danger: Color::Rgb(243, 139, 168),        // Red
                text_primary: Color::Rgb(205, 214, 244),  // Text
                text_secondary: Color::Rgb(186, 194, 222),// Subtext1
                text_muted: Color::Rgb(108, 112, 134),    // Overlay0
                tab_active_bg: Color::Rgb(69, 71, 90),    // Surface1
                tab_active_fg: Color::Rgb(203, 166, 247), // Mauve
                tab_inactive_bg: Color::Rgb(24, 24, 37),  // Mantle
                tab_inactive_fg: Color::Rgb(147, 153, 178),
                selection_bg: Color::Rgb(88, 91, 112),    // Surface2
                selection_fg: Color::White,
            },
            ColorTheme::Nord => ThemePalette {
                bg: Color::Rgb(36, 41, 51),               // Polar Night Dark
                surface: Color::Rgb(46, 52, 64),          // Polar Night
                surface_variant: Color::Rgb(59, 66, 82),  // Polar Night Light
                border_active: Color::Rgb(136, 192, 208), // Frost Ice Blue
                border_inactive: Color::Rgb(76, 86, 106), // Nord border
                accent: Color::Rgb(136, 192, 208),        // Frost Blue
                accent_secondary: Color::Rgb(143, 188, 187), // Frost Sage
                success: Color::Rgb(163, 190, 140),       // Aurora Green
                warning: Color::Rgb(235, 203, 139),       // Aurora Yellow
                danger: Color::Rgb(191, 97, 106),         // Aurora Red
                text_primary: Color::Rgb(236, 239, 244),  // Snow Storm White
                text_secondary: Color::Rgb(216, 222, 233),// Snow Storm
                text_muted: Color::Rgb(94, 105, 126),
                tab_active_bg: Color::Rgb(67, 76, 94),
                tab_active_fg: Color::Rgb(136, 192, 208),
                tab_inactive_bg: Color::Rgb(46, 52, 64),
                tab_inactive_fg: Color::Rgb(148, 163, 184),
                selection_bg: Color::Rgb(67, 76, 94),
                selection_fg: Color::White,
            },
            ColorTheme::GruvboxDark => ThemePalette {
                bg: Color::Rgb(26, 24, 23),               // Gruvbox Warm Hard Dark
                surface: Color::Rgb(40, 36, 34),          // Gruvbox Dark Warm Surface
                surface_variant: Color::Rgb(60, 56, 54),  // Gruvbox Dark1
                border_active: Color::Rgb(250, 189, 47),  // Gruvbox Yellow
                border_inactive: Color::Rgb(80, 73, 69),  // Gruvbox Dark2
                accent: Color::Rgb(250, 189, 47),         // Bright Yellow
                accent_secondary: Color::Rgb(142, 192, 124), // Aqua Green
                success: Color::Rgb(184, 187, 38),        // Bright Green
                warning: Color::Rgb(254, 128, 25),        // Bright Orange
                danger: Color::Rgb(251, 73, 52),          // Bright Red
                text_primary: Color::Rgb(235, 219, 178),  // Light1
                text_secondary: Color::Rgb(213, 196, 161),// Light2
                text_muted: Color::Rgb(146, 131, 116),    // Gray
                tab_active_bg: Color::Rgb(80, 73, 69),
                tab_active_fg: Color::Rgb(250, 189, 47),
                tab_inactive_bg: Color::Rgb(40, 36, 34),
                tab_inactive_fg: Color::Rgb(168, 153, 132),
                selection_bg: Color::Rgb(80, 73, 69),
                selection_fg: Color::White,
            },
            ColorTheme::Cyberpunk => ThemePalette {
                bg: Color::Rgb(13, 4, 30),                // Cyber Void Purple
                surface: Color::Rgb(28, 12, 58),          // Neon Dark Surface
                surface_variant: Color::Rgb(45, 18, 90),  // Neon Violet Variant
                border_active: Color::Rgb(255, 42, 109),  // Hot Neon Pink
                border_inactive: Color::Rgb(54, 34, 102), // Dark Neon Border
                accent: Color::Rgb(5, 217, 232),          // Electric Cyan
                accent_secondary: Color::Rgb(255, 42, 109), // Neon Pink
                success: Color::Rgb(0, 255, 159),         // Cyber Matrix Green
                warning: Color::Rgb(255, 222, 89),        // Neon Gold
                danger: Color::Rgb(255, 0, 85),           // Laser Red
                text_primary: Color::Rgb(255, 255, 255),  // Pure White
                text_secondary: Color::Rgb(213, 200, 245),
                text_muted: Color::Rgb(130, 110, 175),
                tab_active_bg: Color::Rgb(65, 30, 105),
                tab_active_fg: Color::Rgb(5, 217, 232),
                tab_inactive_bg: Color::Rgb(28, 12, 58),
                tab_inactive_fg: Color::Rgb(130, 110, 175),
                selection_bg: Color::Rgb(80, 32, 130),
                selection_fg: Color::White,
            },
            ColorTheme::Dracula => ThemePalette {
                bg: Color::Rgb(30, 31, 41),               // Dracula Void
                surface: Color::Rgb(40, 42, 54),          // Dracula Background
                surface_variant: Color::Rgb(68, 71, 90),  // Dracula Current Line / Selection
                border_active: Color::Rgb(255, 121, 198), // Neon Orchid Pink
                border_inactive: Color::Rgb(68, 71, 90),  // Dracula Slate
                accent: Color::Rgb(189, 147, 249),        // Purple
                accent_secondary: Color::Rgb(139, 233, 253), // Cyan
                success: Color::Rgb(80, 250, 123),        // Neon Green
                warning: Color::Rgb(241, 250, 140),       // Soft Yellow
                danger: Color::Rgb(255, 85, 85),          // Red
                text_primary: Color::Rgb(248, 248, 242),  // Foreground
                text_secondary: Color::Rgb(189, 147, 249),// Purple Tint
                text_muted: Color::Rgb(98, 114, 164),     // Comment Slate
                tab_active_bg: Color::Rgb(68, 71, 90),
                tab_active_fg: Color::Rgb(255, 121, 198),
                tab_inactive_bg: Color::Rgb(40, 42, 54),
                tab_inactive_fg: Color::Rgb(98, 114, 164),
                selection_bg: Color::Rgb(68, 71, 90),
                selection_fg: Color::White,
            },
            ColorTheme::RosePine => ThemePalette {
                bg: Color::Rgb(20, 18, 30),               // Pine Dusk Void
                surface: Color::Rgb(25, 23, 36),          // Pine Base
                surface_variant: Color::Rgb(38, 35, 58),  // Pine Surface
                border_active: Color::Rgb(235, 188, 186), // Rose Blush
                border_inactive: Color::Rgb(57, 53, 82),  // Pine Highlight
                accent: Color::Rgb(235, 188, 186),        // Rose
                accent_secondary: Color::Rgb(196, 167, 231), // Iris
                success: Color::Rgb(156, 207, 216),       // Foam Sea Green
                warning: Color::Rgb(246, 193, 119),       // Gold
                danger: Color::Rgb(235, 111, 146),        // Love Red
                text_primary: Color::Rgb(224, 222, 244),  // Text
                text_secondary: Color::Rgb(144, 140, 170),// Subtle
                text_muted: Color::Rgb(110, 106, 134),    // Muted
                tab_active_bg: Color::Rgb(38, 35, 58),
                tab_active_fg: Color::Rgb(235, 188, 186),
                tab_inactive_bg: Color::Rgb(25, 23, 36),
                tab_inactive_fg: Color::Rgb(110, 106, 134),
                selection_bg: Color::Rgb(49, 44, 76),
                selection_fg: Color::White,
            },
            ColorTheme::MonokaiPro => ThemePalette {
                bg: Color::Rgb(25, 24, 26),               // Volcanic Dark
                surface: Color::Rgb(34, 31, 34),          // Monokai Charcoal
                surface_variant: Color::Rgb(45, 42, 46),  // Monokai Base
                border_active: Color::Rgb(255, 216, 102), // Sublime Yellow
                border_inactive: Color::Rgb(64, 62, 65),  // Border Slate
                accent: Color::Rgb(255, 216, 102),        // Yellow
                accent_secondary: Color::Rgb(255, 97, 136), // Magenta Coral
                success: Color::Rgb(169, 220, 118),       // Mint Green
                warning: Color::Rgb(252, 152, 103),       // Warm Orange
                danger: Color::Rgb(255, 97, 136),         // Coral Red
                text_primary: Color::Rgb(252, 252, 250),  // Pure Ivory
                text_secondary: Color::Rgb(201, 200, 202),
                text_muted: Color::Rgb(114, 112, 114),    // Comment Gray
                tab_active_bg: Color::Rgb(45, 42, 46),
                tab_active_fg: Color::Rgb(255, 216, 102),
                tab_inactive_bg: Color::Rgb(34, 31, 34),
                tab_inactive_fg: Color::Rgb(114, 112, 114),
                selection_bg: Color::Rgb(64, 62, 65),
                selection_fg: Color::White,
            },
            ColorTheme::OneDarkPro => ThemePalette {
                bg: Color::Rgb(30, 34, 42),               // Graphite Dark
                surface: Color::Rgb(40, 44, 52),          // One Dark Surface
                surface_variant: Color::Rgb(44, 49, 58),  // One Dark Highlight
                border_active: Color::Rgb(97, 175, 239),  // Chalk Blue
                border_inactive: Color::Rgb(62, 68, 81),  // Slate Border
                accent: Color::Rgb(97, 175, 239),         // Blue
                accent_secondary: Color::Rgb(198, 120, 221), // Purple
                success: Color::Rgb(152, 195, 121),       // Sage Green
                warning: Color::Rgb(229, 192, 123),       // Warm Amber
                danger: Color::Rgb(224, 108, 117),        // Coral Red
                text_primary: Color::Rgb(171, 178, 191),  // Text
                text_secondary: Color::Rgb(140, 146, 158),
                text_muted: Color::Rgb(92, 99, 112),      // Comment Slate
                tab_active_bg: Color::Rgb(44, 49, 58),
                tab_active_fg: Color::Rgb(97, 175, 239),
                tab_inactive_bg: Color::Rgb(40, 44, 52),
                tab_inactive_fg: Color::Rgb(92, 99, 112),
                selection_bg: Color::Rgb(62, 68, 81),
                selection_fg: Color::White,
            },
        }
    }
}
