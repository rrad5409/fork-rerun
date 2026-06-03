use std::ops::Index;

use egui::{Color32, Sense, Widget};
use re_sdk_types::blueprint::components::NumericBase;
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
    pub fn get(base: NumericBase, c: char) -> Self {
        match (base, c) {
            (NumericBase::Binary, '0') => Self::BinaryZero,
            (NumericBase::Binary, '1') => Self::BinaryOne,
            (NumericBase::Octal, '0') => Self::OctalZero,
            (NumericBase::Octal, '1'..='7') => Self::OctalNum,
            (NumericBase::Hex, '0') => Self::HexZero,
            (NumericBase::Hex, '1'..='9') => Self::HexNum,
            (NumericBase::Hex, 'a'..='f' | 'A'..='F') => Self::HexLetter,
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
    base: NumericBase,
    width: usize,
}

impl Default for RawBytesViewState {
    fn default() -> Self {
        Self {
            range: (0, 0),
            trim: (false, false),
            base: NumericBase::default(),
            width: 40,
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
                    for variant in NumericBase::iter() {
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
    let font_id = ui
        .style()
        .text_styles
        .index(&egui::TextStyle::Monospace)
        .clone();
    let total_height = ui.available_height();

    if entries.is_empty() {
        ui.weak("(no components returned for current query and filters)");
        return;
    }

    for (idx, entry) in entries.into_iter().enumerate() {
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

        let id = ui
            .unique_id()
            .with(&entry.path)
            .with(&entry.component.as_str());

        let heading = format!(
            "{} ({}) [{}/{} bytes]",
            entry.path.ui_string(),
            entry.component.as_str(),
            slice_end - slice_start,
            entry.buf.len(),
        );

        // don't redo during sizing passes, to avoid resize loops
        let resize = !ui.is_sizing_pass()
            && ui.data_mut(|map| {
                let resize = Some(state.width) != map.get_temp(id);
                map.insert_temp(id, state.width);
                resize
            });

        let table = TableDelegate {
            buffer: buf,
            tokens,
            auto_size: resize,
            font_id: font_id.clone(),
            offsets: (slice_start, slice_end),
            base: state.base,
            width: state.width,
        };

        // need to push the id so sibling tables don't conflict with each other
        ui.push_id(id, |ui| {
            // also need to limit the height, else the first table takes it all
            // this is imperfect and leaves some unused space, but the alternative
            // is clipping because the table expands too far
            let h_total = total_height / (entries.len() as f32);
            let h_heading = egui::TextStyle::Heading.resolve(ui.style()).size;
            let h_spacer = 12.0;
            ui.set_min_height(0.0);
            ui.set_max_height(h_total - h_heading - h_spacer);
            // draw a separator line between consecutive entries
            if idx > 0 {
                ui.separator();
            }
            ui.heading(&heading);
            ui.add(table);
        });
    }
}

struct TableDelegate<'a> {
    /// The actual data to display
    buffer: &'a [u8],
    /// Design tokens used for styling the table
    tokens: &'a re_ui::DesignTokens,
    /// Font used for the table.
    ///
    /// Mainly used for column sizing calculations
    font_id: egui::FontId,
    /// Flag to auto-size the column widths this frame.
    /// Required because auto-sizing doesn't update automatically,
    /// and we need to recompute whenever something is modified.
    auto_size: bool,
    /// Offsets to sub-slice `buffer`
    offsets: (usize, usize),
    /// Numeric base to display the bytes in
    base: NumericBase,
    /// How many bytes to display per table row
    width: usize,
}

impl egui_table::TableDelegate for TableDelegate<'_> {
    fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::HeaderCellInfo) {
        ui.colored_label(
            self.tokens.text_strong,
            match cell.group_index {
                0 => "Offset",
                1 => "Raw",
                2 => "Text",
                row => unreachable!("invalid row number {row}"),
            },
        );
    }

    fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
        // slice according to the visualiser's view range
        let slice = &self.buffer[self.offsets.0..self.offsets.1];

        // extract the chunk for this row
        let chunk_start = (cell.row_nr as usize) * self.width;
        let chunk_end = (chunk_start + self.width).min(slice.len());
        let chunk = &slice[chunk_start..chunk_end];

        // PERF(rrad5409): I don't like the repeated allocations here
        // - formatting for row offset
        // - making single chars into strings
        // - creating a new LayoutJob every row
        // - new TextFormat per char

        match cell.col_nr {
            // offset
            0 => {
                egui::Label::new(
                    egui::RichText::new(format!(
                        "{}..{} ", // add a space because tables don't have padding :(
                        self.fmt_offset(self.offsets.0 + chunk_start),
                        self.fmt_offset(self.offsets.0 + chunk_end)
                    ))
                    .monospace(),
                )
                .extend()
                .ui(ui);
            }

            // raw bytes
            1 => {
                let mut job = egui::text::LayoutJob::default();
                for &b in chunk {
                    let s = self.fmt_byte(b);
                    for c in s.chars() {
                        let class = DigitClass::get(self.base, c);
                        job.append(
                            &c.to_string(),
                            0.0,
                            egui::TextFormat::simple(self.font_id.clone(), class.foreground()),
                        );
                    }
                    // space between bytes
                    job.append(
                        " ",
                        0.0,
                        egui::TextFormat::simple(self.font_id.clone(), Color32::TRANSPARENT),
                    );
                }
                // pad to fill the width, so all lines are the same length
                (chunk.len()..self.width).for_each(|_| {
                    job.append(
                        "   ",
                        0.0,
                        egui::TextFormat::simple(self.font_id.clone(), Color32::TRANSPARENT),
                    )
                });
                egui::Label::new(job).extend().ui(ui);
            }

            // text
            2 => {
                let mut job = egui::text::LayoutJob::default();

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
                        egui::text::TextFormat::simple(self.font_id.clone(), class.foreground()),
                    );
                }

                // pad to fill the width, so all lines are the same length
                (chunk.len()..self.width).for_each(|_| {
                    job.append(
                        " ",
                        0.0,
                        egui::TextFormat::simple(self.font_id.clone(), Color32::TRANSPARENT),
                    )
                });
                egui::Label::new(job).extend().ui(ui);
            }
            col => unreachable!("invalid table column {col}"),
        };
    }
}

impl TableDelegate<'_> {
    fn headers(&self) -> impl Into<Vec<egui_table::HeaderRow>> {
        [egui_table::HeaderRow::new(
            self.tokens.table_header_height(),
        )]
    }
    fn columns(&self) -> impl Into<Vec<egui_table::Column>> {
        // auto sizing doesn't seem to work properly unfortunately
        [
            egui_table::Column::default()
                .resizable(self.auto_size)
                .auto_size_this_frame(self.auto_size),
            egui_table::Column::default()
                .resizable(self.auto_size)
                .auto_size_this_frame(self.auto_size),
            egui_table::Column::default()
                .resizable(self.auto_size)
                .auto_size_this_frame(self.auto_size),
        ]
    }
    fn num_rows(&self) -> u64 {
        // avoid underflow on zero-length slicing
        if self.offsets.0 == self.offsets.1 {
            return 0;
        }
        // rounds up in the case of partially filled rows
        ((self.offsets.1 - self.offsets.0 - 1) / self.width) as u64 + 1
    }
    fn fmt_offset(&self, n: usize) -> String {
        match self.base {
            NumericBase::Binary => format!("{n:016b}"),
            NumericBase::Octal => format!("{n:08o}"),
            NumericBase::Decimal => format!("{n:012}"),
            NumericBase::Hex => format!("{n:06X}"),
        }
    }
    fn fmt_byte(&self, n: u8) -> String {
        match self.base {
            NumericBase::Binary => format!("{n:08b}"),
            NumericBase::Octal => format!("{n:03o}"),
            NumericBase::Decimal => format!("{n:03}"),
            NumericBase::Hex => format!("{n:02X}"),
        }
    }
}

impl egui::Widget for &mut TableDelegate<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let table = egui_table::Table::new();
        table
            .num_sticky_cols(1)
            .num_rows(self.num_rows())
            .headers(self.headers())
            .columns(self.columns())
            .show(ui, self)
    }
}

impl egui::Widget for TableDelegate<'_> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        (&mut self).ui(ui)
    }
}

#[test]
fn test_help_view() {
    re_test_context::TestContext::test_help_view(|ctx| RawBytesView.help(ctx));
}
