use std::slice;

use egui::{Label, Sense, Widget};
use itertools::Itertools as _;
use re_sdk_types::{View as _, ViewClassIdentifier};
use re_ui::{Help, OnResponseExt, UiExt, icons};
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

// impl std::fmt::Display for Base {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let s = match self {
//             Base::Binary => "Binary",
//             Base::Octal => "Octal",
//             Base::Hex => "Hex",
//         };
//         f.write_str(s)
//     }
// }

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
        let tokens = ui.tokens();

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

    match entries {
        [] => {
            // We get here if we scroll back time to before the first text document was logged.
            ui.weak("(empty)");
        }
        [RawBytesEntry { blob }] => {
            let buf = &blob.0.0;

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

            let text = egui::RichText::new(format!("{slice:?}")).monospace();

            ui.label("Raw");
            ui.add(Label::new(text).wrap_mode(egui::TextWrapMode::Wrap));
            ui.add_space(8.0);

            // TODO(rrad5409): this would be better using `egui_table::Table`

            egui::Grid::new("grid").show(ui, |ui| {
                // header row
                ui.colored_label(tokens.text_strong, "Offset");
                ui.colored_label(tokens.text_strong, "Data");
                ui.colored_label(tokens.text_strong, "Text");
                ui.end_row();

                ui.style_mut().spacing.item_spacing.x = 0.0;
                // slice our slice into chunks
                for (chunk_idx, chunk) in slice.chunks(state.width).enumerate() {
                    let chunk_start = slice_start + chunk_idx * state.width;
                    let chunk_end = chunk_start + chunk.len() - 1;
                    match state.base {
                        Base::Binary => ui.monospace(format!("{:b}..{:b}", chunk_start, chunk_end)),
                        Base::Octal => ui.monospace(format!("{:o}..{:o}", chunk_start, chunk_end)),
                        Base::Hex => ui.monospace(format!("{:X}..{:X}", chunk_start, chunk_end)),
                    };
                    ui.horizontal(|ui| {
                        for b in chunk {
                            match state.base {
                                Base::Binary => ui.monospace(format!("{b:08b}")),
                                Base::Octal => ui.monospace(format!("{b:03o}")),
                                Base::Hex => ui.monospace(format!("{b:02X}")),
                            };
                        }
                    });
                    ui.horizontal(|ui| {
                        for b in chunk {
                            ui.monospace(str::from_utf8(slice::from_ref(b)).unwrap_or("?"));
                        }
                    });
                    ui.end_row();
                }
            });

            // egui_extras::TableBuilder::new(ui).column(egui_extras::Column::auto())

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
        _ => {
            ui.error_label("TODO (@rrad5409): multiple results handling");
        }
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
