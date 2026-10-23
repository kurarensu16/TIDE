//! Official developer icons (Nerd Fonts) with emoji and clean unicode fallbacks.

use ratatui::style::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconTheme {
    #[default]
    NerdFont,
    Emoji,
    Unicode,
}

impl IconTheme {
    pub fn next(&self) -> Self {
        match self {
            Self::NerdFont => Self::Emoji,
            Self::Emoji => Self::Unicode,
            Self::Unicode => Self::NerdFont,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::NerdFont => "Nerd Fonts",
            Self::Emoji => "Emojis",
            Self::Unicode => "Unicode / Minimal",
        }
    }
}

pub fn file_icon(name: &str, theme: IconTheme) -> (&'static str, Color) {
    match theme {
        IconTheme::NerdFont => {
            if name.ends_with(".rs") {
                (" ", Color::Rgb(249, 115, 22)) // Rust orange
            } else if name.ends_with(".toml") {
                (" ", Color::Rgb(156, 163, 175)) // TOML steel
            } else if name.ends_with(".md") {
                (" ", Color::Rgb(96, 165, 250)) // Markdown blue
            } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".ico") || name.ends_with(".svg") {
                (" ", Color::Rgb(244, 114, 182)) // Image pink
            } else if name.ends_with(".json") {
                (" ", Color::Rgb(250, 204, 21)) // JSON yellow
            } else if name.ends_with(".lock") {
                (" ", Color::Rgb(239, 68, 68)) // Lock red
            } else if name.ends_with(".txt") {
                (" ", Color::Rgb(203, 213, 225)) // Text light slate
            } else if name.ends_with(".sh") || name.ends_with(".bat") || name.ends_with(".ps1") {
                (" ", Color::Rgb(74, 222, 128)) // Shell script green
            } else {
                (" ", Color::Rgb(148, 163, 184)) // Generic file
            }
        }
        IconTheme::Emoji => {
            if name.ends_with(".rs") {
                ("🦀 ", Color::Rgb(249, 115, 22))
            } else if name.ends_with(".toml") {
                ("⚙️ ", Color::Rgb(156, 163, 175))
            } else if name.ends_with(".md") {
                ("📝 ", Color::Rgb(96, 165, 250))
            } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".ico") || name.ends_with(".svg") {
                ("🖼️ ", Color::Rgb(244, 114, 182))
            } else if name.ends_with(".json") {
                ("📋 ", Color::Rgb(250, 204, 21))
            } else if name.ends_with(".lock") {
                ("🔒 ", Color::Rgb(239, 68, 68))
            } else {
                ("📄 ", Color::Rgb(148, 163, 184))
            }
        }
        IconTheme::Unicode => {
            if name.ends_with(".rs") {
                ("rs ", Color::Rgb(249, 115, 22))
            } else if name.ends_with(".toml") {
                ("cfg", Color::Rgb(156, 163, 175))
            } else if name.ends_with(".md") {
                ("md ", Color::Rgb(96, 165, 250))
            } else if name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".ico") || name.ends_with(".svg") {
                ("img", Color::Rgb(244, 114, 182))
            } else if name.ends_with(".json") {
                ("{}", Color::Rgb(250, 204, 21))
            } else if name.ends_with(".lock") {
                ("[x]", Color::Rgb(239, 68, 68))
            } else {
                ("—  ", Color::Rgb(148, 163, 184))
            }
        }
    }
}

pub fn folder_icon(is_expanded: bool, theme: IconTheme) -> (&'static str, Color) {
    match theme {
        IconTheme::NerdFont => {
            if is_expanded {
                (" ", Color::Rgb(56, 189, 248))
            } else {
                (" ", Color::Rgb(56, 189, 248))
            }
        }
        IconTheme::Emoji => {
            if is_expanded {
                ("📂 ", Color::Rgb(234, 179, 8))
            } else {
                ("📁 ", Color::Rgb(234, 179, 8))
            }
        }
        IconTheme::Unicode => {
            if is_expanded {
                ("[-] ", Color::Rgb(56, 189, 248))
            } else {
                ("[+] ", Color::Rgb(56, 189, 248))
            }
        }
    }
}

pub fn arrow_icon(is_dir: bool, is_expanded: bool, theme: IconTheme) -> &'static str {
    if !is_dir {
        return "  ";
    }
    match theme {
        IconTheme::NerdFont => {
            if is_expanded {
                " "
            } else {
                " "
            }
        }
        IconTheme::Emoji => {
            if is_expanded {
                "▼ "
            } else {
                "▶ "
            }
        }
        IconTheme::Unicode => {
            if is_expanded {
                "▾ "
            } else {
                "▸ "
            }
        }
    }
}

pub fn pane_icon(pane: &str, theme: IconTheme) -> &'static str {
    match (pane, theme) {
        ("explorer", IconTheme::NerdFont) => " ",
        ("explorer", IconTheme::Emoji) => "📁 ",
        ("explorer", IconTheme::Unicode) => "≡ ",

        ("preview", IconTheme::NerdFont) => " ",
        ("preview", IconTheme::Emoji) => "📄 ",
        ("preview", IconTheme::Unicode) => "¶ ",

        ("terminal", IconTheme::NerdFont) => " ",
        ("terminal", IconTheme::Emoji) => "💻 ",
        ("terminal", IconTheme::Unicode) => ">_ ",

        ("git", IconTheme::NerdFont) => " ",
        ("git", IconTheme::Emoji) => "🌿 ",
        ("git", IconTheme::Unicode) => "§ ",

        ("agents", IconTheme::NerdFont) => "󰚩 ",
        ("agents", IconTheme::Emoji) => "🤖 ",
        ("agents", IconTheme::Unicode) => "@ ",

        ("bell", IconTheme::NerdFont) => "󰂚 ",
        ("bell", IconTheme::Emoji) => "🔔 ",
        ("bell", IconTheme::Unicode) => "! ",

        ("wave", IconTheme::NerdFont) => "󰘬 ",
        ("wave", IconTheme::Emoji) => "🌊 ",
        ("wave", IconTheme::Unicode) => "≈~ ",

        _ => "",
    }
}

pub fn action_icon(action: &str, theme: IconTheme) -> &'static str {
    match (action, theme) {
        ("search", IconTheme::NerdFont) => " ",
        ("search", IconTheme::Emoji) => "🔍 ",
        ("search", IconTheme::Unicode) => "? ",

        ("palette", IconTheme::NerdFont) => "󰘳 ",
        ("palette", IconTheme::Emoji) => "🔍 ",
        ("palette", IconTheme::Unicode) => "> ",

        ("terminal", IconTheme::NerdFont) => " ",
        ("terminal", IconTheme::Emoji) => "💻 ",
        ("terminal", IconTheme::Unicode) => ">_ ",

        ("agent", IconTheme::NerdFont) => "󰚩 ",
        ("agent", IconTheme::Emoji) => "🤖 ",
        ("agent", IconTheme::Unicode) => "@ ",

        ("theme", IconTheme::NerdFont) => "󰏘 ",
        ("theme", IconTheme::Emoji) => "🎨 ",
        ("theme", IconTheme::Unicode) => "# ",

        ("commit", IconTheme::NerdFont) => "󰊢 ",
        ("commit", IconTheme::Emoji) => "📦 ",
        ("commit", IconTheme::Unicode) => "§ ",

        ("branch", IconTheme::NerdFont) => " ",
        ("branch", IconTheme::Emoji) => "🌿 ",
        ("branch", IconTheme::Unicode) => "& ",

        ("stash", IconTheme::NerdFont) => "󰉉 ",
        ("stash", IconTheme::Emoji) => "📦 ",
        ("stash", IconTheme::Unicode) => "$ ",

        ("diff", IconTheme::NerdFont) => " ",
        ("diff", IconTheme::Emoji) => "🔍 ",
        ("diff", IconTheme::Unicode) => "± ",

        ("file", IconTheme::NerdFont) => " ",
        ("file", IconTheme::Emoji) => "📄 ",
        ("file", IconTheme::Unicode) => "— ",

        ("folder", IconTheme::NerdFont) => " ",
        ("folder", IconTheme::Emoji) => "📁 ",
        ("folder", IconTheme::Unicode) => "[+] ",

        ("new_item", IconTheme::NerdFont) => " ",
        ("new_item", IconTheme::Emoji) => "📄 ",
        ("new_item", IconTheme::Unicode) => "+ ",

        ("rename", IconTheme::NerdFont) => "󰑕 ",
        ("rename", IconTheme::Emoji) => "✏️ ",
        ("rename", IconTheme::Unicode) => "~ ",

        ("delete", IconTheme::NerdFont) => " ",
        ("delete", IconTheme::Emoji) => "🗑️ ",
        ("delete", IconTheme::Unicode) => "x ",

        ("save", IconTheme::NerdFont) => "󰆓 ",
        ("save", IconTheme::Emoji) => "💾 ",
        ("save", IconTheme::Unicode) => "^ ",

        ("undo", IconTheme::NerdFont) => " ",
        ("undo", IconTheme::Emoji) => "⤺ ",
        ("undo", IconTheme::Unicode) => "⤺ ",

        ("redo", IconTheme::NerdFont) => " ",
        ("redo", IconTheme::Emoji) => "⤻ ",
        ("redo", IconTheme::Unicode) => "⤻ ",

        ("duplicate", IconTheme::NerdFont) => "󰆏 ",
        ("duplicate", IconTheme::Emoji) => "📋 ",
        ("duplicate", IconTheme::Unicode) => "++ ",

        ("comment", IconTheme::NerdFont) => "󰅺 ",
        ("comment", IconTheme::Emoji) => "💬 ",
        ("comment", IconTheme::Unicode) => "// ",

        ("push", IconTheme::NerdFont) => "󰛃 ",
        ("push", IconTheme::Emoji) => "🚀 ",
        ("push", IconTheme::Unicode) => ">> ",

        ("pull", IconTheme::NerdFont) => "󰛀 ",
        ("pull", IconTheme::Emoji) => "📥 ",
        ("pull", IconTheme::Unicode) => "<< ",

        ("vscode", IconTheme::NerdFont) => "󰨞 ",
        ("vscode", IconTheme::Emoji) => "💻 ",
        ("vscode", IconTheme::Unicode) => "[>] ",

        ("scroll", IconTheme::NerdFont) => "󰋚 ",
        ("scroll", IconTheme::Emoji) => "📜 ",
        ("scroll", IconTheme::Unicode) => ":: ",

        ("zoom", IconTheme::NerdFont) => "󰍉 ",
        ("zoom", IconTheme::Emoji) => "🔍 ",
        ("zoom", IconTheme::Unicode) => "* ",

        ("refresh", IconTheme::NerdFont) => " ",
        ("refresh", IconTheme::Emoji) => "🔄 ",
        ("refresh", IconTheme::Unicode) => "~ ",

        ("stop", IconTheme::NerdFont) => " ",
        ("stop", IconTheme::Emoji) => "⏹️ ",
        ("stop", IconTheme::Unicode) => "[] ",

        ("maximize", IconTheme::NerdFont) => "󰊓 ",
        ("maximize", IconTheme::Emoji) => "🗖 ",
        ("maximize", IconTheme::Unicode) => "[^] ",

        ("quit", IconTheme::NerdFont) => "󰗼 ",
        ("quit", IconTheme::Emoji) => "🚪 ",
        ("quit", IconTheme::Unicode) => "q ",

        _ => "",
    }
}

pub fn shell_icon(shell: &str, theme: IconTheme) -> &'static str {
    match (shell, theme) {
        ("cmd", IconTheme::NerdFont) => " ",
        ("cmd", IconTheme::Emoji) => "💻 ",
        ("cmd", IconTheme::Unicode) => "cmd ",

        ("powershell", IconTheme::NerdFont) => "󰨊 ",
        ("powershell", IconTheme::Emoji) => "⚡ ",
        ("powershell", IconTheme::Unicode) => "ps ",

        ("bash", IconTheme::NerdFont) => " ",
        ("bash", IconTheme::Emoji) => "🐚 ",
        ("bash", IconTheme::Unicode) => "sh ",

        ("wsl", IconTheme::NerdFont) => " ",
        ("wsl", IconTheme::Emoji) => "🐧 ",
        ("wsl", IconTheme::Unicode) => "wsl ",

        _ => " ",
    }
}
