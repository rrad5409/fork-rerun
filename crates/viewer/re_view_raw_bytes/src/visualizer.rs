use re_log_types::EntityPath;
use re_log_types::external::arrow;
use re_sdk_types::Archetype as _;
use re_sdk_types::Loggable as _;
use re_sdk_types::archetypes;
use re_sdk_types::components;
use re_viewer_context::{
    IdentifiedViewSystem, SingleRequiredComponentConstraint, ViewContext, ViewContextCollection,
    ViewQuery, ViewSystemExecutionError, VisualizerExecutionOutput, VisualizerQueryInfo,
    VisualizerSystem,
};

// ---

#[derive(Debug, Clone)]
pub struct RawBytesEntry {
    /// The entity that this entry was sourced from
    pub path: EntityPath,
    /// The component that we read this entry from
    pub component: re_sdk_types::ComponentIdentifier,
    /// The raw data of this entry
    pub buf: arrow::buffer::Buffer,
}

/// A text scene, with everything needed to render it.
#[derive(Default)]
pub struct RawBytesSystem;

impl IdentifiedViewSystem for RawBytesSystem {
    fn identifier() -> re_viewer_context::ViewSystemIdentifier {
        "RawBytes".into()
    }
}

impl VisualizerSystem for RawBytesSystem {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo {
            relevant_archetype: Some(archetypes::RawBytes::name()),
            constraints: SingleRequiredComponentConstraint::new::<components::Blob>(
                &archetypes::RawBytes::descriptor_blob(),
            )
            .with_additional_physical_types([
                components::Text::arrow_datatype(),
                components::ImageBuffer::arrow_datatype(),
                components::VideoSample::arrow_datatype(),
            ])
            .into(),
            // hehe: re_sdk_types::reflection::generate_reflection().unwrap().components
            queried: archetypes::RawBytes::all_components()
                .into_iter()
                .cloned()
                .collect(),
        }
    }

    fn execute(
        &self,
        ctx: &ViewContext<'_>,
        view_query: &ViewQuery<'_>,
        _context_systems: &ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let mut entries: Vec<RawBytesEntry> = Vec::new();

        for (data_result, instruction) in
            view_query.iter_visualizer_instruction_for(Self::identifier())
        {
            ctx.egui_ctx().debug_text(format!("{data_result:#?}"));

            let results = re_view::latest_at_with_blueprint_resolved_data(
                ctx,
                None,
                &view_query.latest_at_query(),
                data_result,
                archetypes::RawBytes::all_component_identifiers(),
                Some(instruction),
            );

            // TODO: should we use `results.get_unit_chunk()` instead of `.get_mono()`?
            // does that allow for multiple components of the same type per entity?

            if let Some(buf) = results
                .get_mono::<components::Blob>(archetypes::RawBytes::descriptor_blob().component)
                .map(|blob| blob.0.0.into_inner())
            {
                entries.push(RawBytesEntry {
                    buf,
                    component: archetypes::RawBytes::descriptor_blob().component,
                    path: results.entity_path().clone(),
                });
            };
        }

        Ok(VisualizerExecutionOutput::default().with_visualizer_data(entries))
    }
}
