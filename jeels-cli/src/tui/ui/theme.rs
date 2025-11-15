use ratatui::style::Color;

pub struct NeonTheme;

impl NeonTheme {
    // Neon purple colors
    pub const PURPLE_BRIGHT: Color = Color::Rgb(186, 85, 211);  // MediumOrchid
    pub const PURPLE_NEON: Color = Color::Rgb(138, 43, 226);   // BlueViolet
    pub const PURPLE_DARK: Color = Color::Rgb(75, 0, 130);     // Indigo
    
    // Neon green colors
    pub const GREEN_NEON: Color = Color::Rgb(57, 255, 20);     // Bright neon green
    pub const GREEN_BRIGHT: Color = Color::Rgb(0, 255, 127);   // SpringGreen
    pub const GREEN_DARK: Color = Color::Rgb(0, 200, 100);
    
    // Accent colors
    pub const MAGENTA: Color = Color::Rgb(255, 20, 147);       // DeepPink
    pub const CYAN: Color = Color::Rgb(0, 255, 255);           // Cyan
    pub const YELLOW: Color = Color::Rgb(255, 255, 0);         // Yellow
    
    // Text colors
    pub const TEXT_BRIGHT: Color = Color::Rgb(255, 255, 255);
    pub const TEXT_DIM: Color = Color::Rgb(180, 180, 200);
    pub const TEXT_DARK: Color = Color::Rgb(100, 100, 120);
    
    // Background
    pub const BG_DARK: Color = Color::Rgb(10, 10, 20);
    pub const BG_HIGHLIGHT: Color = Color::Rgb(30, 10, 40);
    
    // Sparkle symbols
    pub const SPARKLE: &'static str = "✨";
    pub const STAR: &'static str = "⭐";
    pub const SPARKLES: &'static str = "💫";
    pub const DIAMOND: &'static str = "💎";
}

