//! Applies the kit's style vars and colour slots to an `imgui::Style`, and
//! exposes button variants as colour stacks.

use imgui::{ColorStackToken, Context, Style, StyleColor, Ui};

use crate::tokens::{color, size, space, Rgba};

/// Writes every style var and colour of the kit into `style`.
/// Call once after creating the context, before loading fonts is fine too.
pub fn apply(style: &mut Style) {
    // --- layout vars (artboard 02 · table 04) --------------------------
    style.window_padding = [space::XL, space::XL];
    style.frame_padding = [size::PAD_X, size::PAD_Y];
    style.item_spacing = [space::S, space::S];
    style.item_inner_spacing = [space::S, space::XS];
    style.cell_padding = [size::PAD_X, size::PAD_Y];
    style.indent_spacing = space::L;
    style.scrollbar_size = size::SCROLLBAR;
    style.grab_min_size = size::KNOB;
    style.columns_min_spacing = space::L;

    style.window_rounding = 0.0;
    style.child_rounding = 0.0;
    style.frame_rounding = size::RADIUS;
    style.popup_rounding = size::RADIUS;
    style.tab_rounding = size::RADIUS;
    style.scrollbar_rounding = size::RADIUS;
    style.grab_rounding = size::KNOB / 2.0;

    style.window_border_size = size::BORDER;
    style.child_border_size = 0.0;
    style.frame_border_size = size::BORDER;
    style.popup_border_size = size::BORDER;
    style.tab_border_size = 0.0;

    style.window_title_align = [0.0, 0.5];
    style.button_text_align = [0.5, 0.5];
    style.selectable_text_align = [0.0, 0.5];
    style.window_min_size = [960.0, 640.0];
    style.anti_aliased_lines = true;
    style.anti_aliased_fill = true;

    // --- colour slots (artboard 01 · section 01) -----------------------
    use StyleColor as C;
    let c = &mut style.colors;
    c[C::Text as usize] = color::FG;
    c[C::TextDisabled as usize] = color::FG3;
    c[C::WindowBg as usize] = color::BG0;
    c[C::ChildBg as usize] = color::TRANSPARENT;
    c[C::PopupBg as usize] = color::BG2;
    c[C::Border as usize] = color::LINE;
    c[C::BorderShadow as usize] = color::TRANSPARENT;
    c[C::FrameBg as usize] = color::BG1;
    c[C::FrameBgHovered as usize] = color::BG3;
    c[C::FrameBgActive as usize] = color::BG4;
    c[C::TitleBg as usize] = color::BG2;
    c[C::TitleBgActive as usize] = color::BG2;
    c[C::TitleBgCollapsed as usize] = color::BG2;
    c[C::MenuBarBg as usize] = color::BG2;
    c[C::ScrollbarBg as usize] = color::BG0;
    c[C::ScrollbarGrab as usize] = color::LINE2;
    c[C::ScrollbarGrabHovered as usize] = color::FG3;
    c[C::ScrollbarGrabActive as usize] = color::FG2;
    c[C::CheckMark as usize] = color::ACCENT;
    c[C::SliderGrab as usize] = color::ACCENT;
    c[C::SliderGrabActive as usize] = color::ACCENT_HOVER;
    // Secondary button is the default button.
    c[C::Button as usize] = color::BG2;
    c[C::ButtonHovered as usize] = color::BG4;
    c[C::ButtonActive as usize] = hex_dark();
    c[C::Header as usize] = color::SEL;
    c[C::HeaderHovered as usize] = color::BG3;
    c[C::HeaderActive as usize] = color::BG4;
    c[C::Separator as usize] = color::LINE;
    c[C::SeparatorHovered as usize] = color::LINE2;
    c[C::SeparatorActive as usize] = color::FG3;
    c[C::ResizeGrip as usize] = color::TRANSPARENT;
    c[C::ResizeGripHovered as usize] = color::LINE2;
    c[C::ResizeGripActive as usize] = color::FG3;
    c[C::Tab as usize] = color::TRANSPARENT;
    c[C::TabHovered as usize] = color::BG2;
    c[C::TabActive as usize] = color::BG3;
    c[C::TabUnfocused as usize] = color::TRANSPARENT;
    c[C::TabUnfocusedActive as usize] = color::BG3;
    c[C::PlotLines as usize] = color::FG2;
    c[C::PlotLinesHovered as usize] = color::ACCENT;
    c[C::PlotHistogram as usize] = color::LINE2;
    c[C::PlotHistogramHovered as usize] = color::ACCENT;
    c[C::TableHeaderBg as usize] = color::BG2;
    c[C::TableBorderStrong as usize] = color::LINE;
    c[C::TableBorderLight as usize] = color::LINE;
    c[C::TableRowBg as usize] = color::TRANSPARENT;
    c[C::TableRowBgAlt as usize] = crate::tokens::hex(0x151515);
    c[C::TextSelectedBg as usize] = color::SEL;
    c[C::DragDropTarget as usize] = color::ACCENT;
    c[C::NavHighlight as usize] = color::ACCENT;
    c[C::NavWindowingHighlight as usize] = color::ACCENT;
    c[C::NavWindowingDimBg as usize] = color::DIM;
    c[C::ModalWindowDimBg as usize] = color::DIM;
}

/// Convenience: applies the theme to a context.
pub fn apply_to(ctx: &mut Context) {
    apply(ctx.style_mut());
}

const fn hex_dark() -> Rgba {
    crate::tokens::hex(0x161616)
}

/// Button variants of artboard 03 · section 01.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ButtonKind {
    Primary,
    #[default]
    Secondary,
    Danger,
    Ghost,
}

/// The five colours a button variant needs.
#[derive(Clone, Copy, Debug)]
pub struct ButtonColors {
    pub bg: Rgba,
    pub hover: Rgba,
    pub active: Rgba,
    pub text: Rgba,
    pub border: Rgba,
}

impl ButtonKind {
    pub const fn colors(self) -> ButtonColors {
        match self {
            ButtonKind::Primary => ButtonColors {
                bg: color::ACCENT,
                hover: color::ACCENT_HOVER,
                active: color::ACCENT_ACTIVE,
                text: color::BG0,
                border: color::ACCENT,
            },
            ButtonKind::Secondary => ButtonColors {
                bg: color::BG2,
                hover: color::BG4,
                active: hex_dark(),
                text: color::FG,
                border: color::LINE2,
            },
            ButtonKind::Danger => ButtonColors {
                bg: color::DANGER_BG,
                hover: color::DANGER_BG_HOVER,
                active: color::ERR,
                text: color::ERR,
                border: color::DANGER_BORDER,
            },
            ButtonKind::Ghost => ButtonColors {
                bg: color::TRANSPARENT,
                hover: color::BG3,
                active: color::BG2,
                text: color::FG2,
                border: color::TRANSPARENT,
            },
        }
    }

    /// Pushes the variant's colours. Keep the returned guard alive while the
    /// button is drawn; dropping it pops the colours again.
    pub fn push<'ui>(self, ui: &'ui Ui) -> ButtonStyle<'ui> {
        let c = self.colors();
        ButtonStyle {
            _tokens: [
                ui.push_style_color(StyleColor::Button, c.bg),
                ui.push_style_color(StyleColor::ButtonHovered, c.hover),
                ui.push_style_color(StyleColor::ButtonActive, c.active),
                ui.push_style_color(StyleColor::Text, c.text),
                ui.push_style_color(StyleColor::Border, c.border),
            ],
        }
    }
}

/// Guard returned by [`ButtonKind::push`].
pub struct ButtonStyle<'ui> {
    _tokens: [ColorStackToken<'ui>; 5],
}
