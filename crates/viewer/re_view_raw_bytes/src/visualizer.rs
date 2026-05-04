use re_chunk_store::LatestAtQuery;
use re_sdk_types::Archetype as _;
use re_sdk_types::archetypes;
use re_sdk_types::components;
use re_view::DataResultQuery as _;
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewContextCollection, ViewQuery, ViewSystemExecutionError,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};

// ---

#[derive(Debug, Clone)]
pub struct RawBytesEntry {
    pub blob: components::Blob,
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
        VisualizerQueryInfo::single_required_component::<components::Blob>(
            &archetypes::RawBytes::descriptor_blob(),
            &archetypes::RawBytes::all_components(),
        )
    }

    fn execute(
        &self,
        ctx: &ViewContext<'_>,
        view_query: &ViewQuery<'_>,
        _context_systems: &ViewContextCollection,
    ) -> Result<VisualizerExecutionOutput, ViewSystemExecutionError> {
        let timeline_query = LatestAtQuery::new(view_query.timeline, view_query.latest_at);

        let mut text_entries = Vec::new();

        for (data_result, instruction) in
            view_query.iter_visualizer_instruction_for(Self::identifier())
        {
            let results = data_result
                .latest_at_with_blueprint_resolved_data::<archetypes::RawBytes>(
                    ctx,
                    &timeline_query,
                    Some(instruction),
                );

            let Some(blob) = results
                .get_mono::<components::Blob>(archetypes::RawBytes::descriptor_blob().component)
            else {
                continue;
            };
            text_entries.push(RawBytesEntry { blob: blob.clone() });
        }

        Ok(VisualizerExecutionOutput::default().with_visualizer_data(text_entries))
    }
}
