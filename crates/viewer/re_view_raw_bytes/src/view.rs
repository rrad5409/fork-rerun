use std::ops::Index;

use egui::{Sense, Widget};
use re_sdk_types::{View as _, ViewClassIdentifier};
use re_ui::{Help, UiExt};
use re_viewer_context::external::re_log_types::EntityPath;
use re_viewer_context::{
    IdentifiedViewSystem as _, Item, SystemCommand, SystemCommandSender as _, ViewClass,
    ViewClassRegistryError, ViewId, ViewQuery, ViewState, ViewStateExt as _,
    ViewSystemExecutionError, ViewerContext, suggest_view_for_each_entity,
};
use strum::IntoEnumIterator;

use crate::visualizer::{RawBytesEntry, RawBytesSystem};

// TODO: move this to component defaults
#[derive(
    Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumIter,
)]
#[repr(u8)]
enum Base {
    Binary = 2,
    Octal = 8,
    #[default]
    Hex = 16,
}

/// Represents a class of characters, used to distinguish them when rendering
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumIter)]
enum CharacterClass {
    Upper,
    Lower,
    Numeric,
    Punctuation,
    Control,
    Other,
}

impl CharacterClass {
    pub fn get(c: char) -> Self {
        if c.is_digit(10) {
            Self::Numeric
        } else if c.is_uppercase() {
            Self::Upper
        } else if c.is_lowercase() {
            Self::Lower
        } else if c.is_ascii_punctuation() {
            Self::Punctuation
        } else if c.is_control() || c.is_ascii_control() {
            Self::Control
        } else {
            Self::Other
        }
    }
    /// Gets the foreground colour for this class
    pub fn foreground(self) -> egui::Color32 {
        match self {
            Self::Upper => egui::Color32::WHITE,
            Self::Lower => egui::Color32::from_gray(0xC8),
            Self::Numeric => egui::Color32::from_rgb(0x50, 0x70, 0xF0),
            Self::Punctuation => egui::Color32::GREEN,
            Self::Control => egui::Color32::ORANGE,
            Self::Other => egui::Color32::MAGENTA,
        }
    }
}

/// Represents a class of digits, used to render them in different colours.
///
/// E.g. for binary we have `DigitClass::BinaryZero` and `DigitClass::BinaryOne`
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumIter)]
enum DigitClass {
    BinaryZero,
    BinaryOne,
    OctalZero,
    OctalNum,
    HexZero,
    HexNum,
    HexLetter,
    Unknown,
}

impl DigitClass {
    pub fn get(base: Base, c: char) -> Self {
        match (base, c) {
            (Base::Binary, '0') => Self::BinaryZero,
            (Base::Binary, '1') => Self::BinaryOne,
            (Base::Octal, '0') => Self::OctalZero,
            (Base::Octal, '1'..='7') => Self::OctalNum,
            (Base::Hex, '0') => Self::HexZero,
            (Base::Hex, '1'..='9') => Self::HexNum,
            (Base::Hex, 'a'..='f' | 'A'..='F') => Self::HexLetter,
            _ => Self::Unknown,
        }
    }

    /// Gets the foreground colour for this class
    pub fn foreground(self) -> egui::Color32 {
        match self {
            // strong contrast
            Self::BinaryZero => egui::Color32::GRAY,
            Self::BinaryOne => egui::Color32::WHITE,
            // lower contrast to not be overwhelming
            Self::OctalZero | Self::HexZero => egui::Color32::LIGHT_GRAY,
            Self::OctalNum | Self::HexNum => egui::Color32::LIGHT_GREEN,
            Self::HexLetter => egui::Color32::from_rgb(0x50, 0x70, 0xF0),
            // fallback
            Self::Unknown => egui::Color32::LIGHT_RED,
        }
    }
}

pub struct RawBytesViewState {
    range: (usize, usize),
    trim: (bool, bool),
    base: Base,
    width: usize,
}

impl Default for RawBytesViewState {
    fn default() -> Self {
        Self {
            range: (usize::MIN, usize::MAX),
            trim: (false, false),
            base: Base::default(),
            width: 16,
        }
    }
}

impl ViewState for RawBytesViewState {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Default)]
pub struct RawBytesView;

type ViewType = re_sdk_types::blueprint::views::RawBytesView;

impl ViewClass for RawBytesView {
    fn identifier() -> ViewClassIdentifier {
        ViewType::identifier()
    }

    fn display_name(&self) -> &'static str {
        "Raw bytes"
    }

    fn icon(&self) -> &'static re_ui::Icon {
        &re_ui::icons::VIEW_BINARY
    }

    fn help(&self, _os: egui::os::OperatingSystem) -> Help {
        Help::new("Raw bytes view")
            .docs_link("https://rerun.io/docs/reference/types/views/raw_bytes_view")
            .markdown("**TODO: This view is still wip**")
    }

    fn on_register(
        &self,
        system_registry: &mut re_viewer_context::ViewSystemRegistrator<'_>,
    ) -> Result<(), ViewClassRegistryError> {
        system_registry.register_visualizer::<RawBytesSystem>()
    }

    fn new_state(&self) -> Box<dyn ViewState> {
        Box::<RawBytesViewState>::default()
    }

    fn layout_priority(&self) -> re_viewer_context::ViewClassLayoutPriority {
        re_viewer_context::ViewClassLayoutPriority::Low
    }

    fn selection_ui(
        &self,
        _ctx: &ViewerContext<'_>,
        ui: &mut egui::Ui,
        state: &mut dyn ViewState,
        _space_origin: &EntityPath,
        _view_id: ViewId,
    ) -> Result<(), ViewSystemExecutionError> {
        let state = state.downcast_mut::<RawBytesViewState>()?;

        ui.warning_label("TODO: selected ui");

        ui.collapsing_header("Misc", true, |ui| {
            ui.selection_grid("misc").show(ui, |ui| {
                ui.label("Base");
                ui.drop_down_menu("base", state.base.to_string(), |ui| {
                    for variant in Base::iter() {
                        ui.selectable_value(&mut state.base, variant, variant.to_string());
                    }
                });
                ui.end_row();

                ui.label("Chunk Width");
                egui::DragValue::new(&mut state.width)
                    .clamp_existing_to_range(true)
                    .range(1..=1024)
                    .ui(ui);
                ui.end_row();
            });
        });

        ui.collapsing_header("Range", true, |ui| {
            ui.selection_grid("range").show(ui, |ui| {
                ui.label("Clamp");
                ui.re_checkbox(&mut state.trim.0, "Start");
                ui.re_checkbox(&mut state.trim.1, "End");
                ui.end_row();

                ui.add_enabled_ui(state.trim.0, |ui| {
                    ui.label("Start");
                    egui::DragValue::new(&mut state.range.0).ui(ui);
                    increment_buttons(ui, &mut state.range.0, 0, usize::MAX);
                });
                ui.end_row();

                ui.add_enabled_ui(state.trim.1, |ui| {
                    ui.label("End");
                    egui::DragValue::new(&mut state.range.1).ui(ui);
                    increment_buttons(ui, &mut state.range.1, 0, usize::MAX);
                });
                ui.end_row();
            });
        });

        Ok(())
    }

    fn spawn_heuristics(
        &self,
        ctx: &ViewerContext<'_>,
        include_entity: &dyn Fn(&EntityPath) -> bool,
    ) -> re_viewer_context::ViewSpawnHeuristics {
        re_tracing::profile_function!();
        // By default spawn a view for every entity.
        suggest_view_for_each_entity::<RawBytesSystem>(ctx, include_entity)
    }

    fn ui(
        &self,
        ctx: &ViewerContext<'_>,
        _missing_chunk_reporter: &re_viewer_context::MissingChunkReporter,
        ui: &mut egui::Ui,
        state: &mut dyn ViewState,
        query: &ViewQuery<'_>,
        system_output: re_viewer_context::SystemExecutionOutput,
    ) -> Result<(), ViewSystemExecutionError> {
        let tokens = ui.tokens();
        let state = state.downcast_mut::<RawBytesViewState>()?;
        let entries =
            system_output.visualizer_data::<Vec<RawBytesEntry>>(RawBytesSystem::identifier())?;

        let frame = egui::Frame::new().inner_margin(tokens.view_padding());
        let response = frame
            .show(ui, |ui| {
                let inner_ui_builder = egui::UiBuilder::new()
                    .layout(egui::Layout::top_down(egui::Align::LEFT))
                    .sense(Sense::click());
                ui.scope_builder(inner_ui_builder, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show(ui, |ui| raw_bytes_ui(ui, state, entries));

                    ui.response()
                })
                .inner
            })
            .inner;

        // Since we want the view to be hoverable / clickable when the pointer is over a label
        // (and we want selectable labels), we need to work around egui's interactions here.
        // Since `rect_contains_pointer` checks for the layer id, this shouldn't cause any problems
        // with popups / modals.
        let hovered = ui.ctx().rect_contains_pointer(ui.layer_id(), response.rect);
        let clicked = hovered && ui.input(|i| i.pointer.primary_pressed());

        if hovered {
            ctx.selection_state().set_hovered(Item::View(query.view_id));
        }

        if clicked {
            ctx.command_sender()
                .send_system(SystemCommand::set_selection(Item::View(query.view_id)));
        }

        Ok(())
    }
}

fn increment_buttons<
    N: emath::Numeric + num_traits::Num + num_traits::PrimInt + core::fmt::Display,
>(
    ui: &mut egui::Ui,
    val: &mut N,
    min: N,
    max: N,
) {
    let mut step = N::one();
    if ui.input(|i| i.modifiers.command) {
        step = step.unsigned_shl(3);
    }
    if ui.input(|i| i.modifiers.shift) {
        step = step.unsigned_shl(7);
    }

    if ui.small_button(format!("+{step}")).clicked() {
        *val = val.saturating_add(step);
    }
    if ui.small_button(format!("-{step}")).clicked() {
        *val = val.saturating_sub(step);
    }

    *val = (*val).clamp(min, max);
}

fn raw_bytes_ui(ui: &mut egui::Ui, state: &mut RawBytesViewState, entries: &[RawBytesEntry]) {
    let tokens = ui.tokens();

    if entries.is_empty() {
        ui.weak("(nothing to show here)");
        return;
    }

    let fmt_usize = |n: usize| match state.base {
        Base::Binary => format!("{n:08b}"),
        Base::Octal => format!("{n:03o}"),
        Base::Hex => format!("{n:02X}"),
    };
    let fmt_u8 = |n: u8| match state.base {
        Base::Binary => format!("{n:08b}"),
        Base::Octal => format!("{n:03o}"),
        Base::Hex => format!("{n:02X}"),
    };

    for entry in entries {
        let buf = &entry.buf;

        // slice the buffer for selected offset range
        let slice_start = if state.trim.0 {
            state.range.0.min(buf.len())
        } else {
            0
        };
        let slice_end = if state.trim.1 {
            state.range.1.min(buf.len()).max(slice_start)
        } else {
            buf.len()
        };
        let slice = &buf[slice_start..slice_end];

        // PERF(rrad5409): this would be better using `egui_table::Table`
        ui.heading(entry.path.ui_string());
        egui_extras::TableBuilder::new(ui)
            .id_salt(&entry.path)
            .columns(egui_extras::Column::auto().clip(false), 3)
            .resizable(false)
            .striped(false)
            .header(tokens.table_header_height(), |mut row| {
                row.col(|ui| {
                    ui.colored_label(tokens.text_strong, "Offset");
                });
                row.col(|ui| {
                    ui.colored_label(tokens.text_strong, "Raw");
                });
                row.col(|ui| {
                    ui.colored_label(tokens.text_strong, "Text");
                });
            })
            .body(|body| {
                if slice.is_empty() {
                    return;
                }
                body.rows(
                    tokens.table_row_height(re_ui::TableStyle::Dense),
                    ((slice.len() - 1) / state.width) + 1,
                    // PERF(rrad5409): does egui automatically only run render visible rows?
                    |mut row| {
                        let chunk_start = row.index() * state.width;
                        let chunk_end = (chunk_start + state.width).min(slice.len());
                        let chunk = &slice[chunk_start..chunk_end];

                        row.set_overline(row.index() % 8 == 0);

                        // PERF(rrad5409): I don't like the repeated allocations here
                        // - formatting for row offset
                        // - making single chars into strings
                        // - creating a new LayoutJob every row
                        // - new TextFormat per char

                        // offset
                        row.col(|ui| {
                            egui::Label::new(
                                egui::RichText::new(format!(
                                    "{}..{}",
                                    fmt_usize(slice_start + chunk_start),
                                    fmt_usize(slice_start + chunk_end)
                                ))
                                .monospace(),
                            )
                            .extend()
                            .ui(ui);
                        });

                        // raw
                        row.col(|ui| {
                            let mut job = egui::text::LayoutJob::default();
                            let font = ui.style().text_styles.index(&egui::TextStyle::Monospace);

                            for &b in chunk {
                                let s = fmt_u8(b);
                                for c in s.chars() {
                                    let class = DigitClass::get(state.base, c);
                                    job.append(
                                        &c.to_string(),
                                        0.0,
                                        egui::TextFormat::simple(font.clone(), class.foreground()),
                                    );
                                }
                                job.append(" ", 0.0, egui::TextFormat::default());
                            }

                            egui::Label::new(job).extend().ui(ui);
                        });

                        // text
                        row.col(|ui| {
                            let mut job = egui::text::LayoutJob::default();
                            let font = ui.style().text_styles.index(&egui::TextStyle::Monospace);

                            for &b in chunk {
                                let mut c = char::from_u32(b as u32).unwrap_or('?');
                                let class = CharacterClass::get(c);
                                // control characters make the spacing funky, so we replace them
                                if c.is_ascii_control() {
                                    c = '.';
                                }
                                job.append(
                                    &c.to_string(),
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: font.clone(),
                                        color: class.foreground(),
                                        ..Default::default()
                                    },
                                );
                            }

                            egui::Label::new(job).extend().ui(ui);
                        });
                    },
                );
            });

        // let mut table = TableDelegate {
        //     buffer: buf,
        //     offsets: (slice_start, slice_end),
        //     base: state.base,
        //     width: state.width,
        // };
        // egui_table::Table::new()
        //     // .num_sticky_cols(state.width * 2 + 3)
        //     .columns((0..))
        //     .num_rows(((slice.len() - 1) / state.width) as u64 + 1)
        //     .show(ui, &mut table);
    }
}

// struct TableDelegate<'a> {
//     buffer: &'a [u8],
//     offsets: (usize, usize),
//     base: Base,
//     width: usize,
// }
//
// impl egui_table::TableDelegate for TableDelegate<'_> {
//     fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::HeaderCellInfo) {
//         ui.label(format!("[{}]", cell.row_nr));
//     }
//
//     fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
//         ui.label(format!("{}x{}", cell.col_nr, cell.row_nr));
//     }
// }

#[test]
fn test_help_view() {
    re_test_context::TestContext::test_help_view(|ctx| RawBytesView.help(ctx));
}
