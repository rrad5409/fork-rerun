use itertools::Itertools;
use re_log_types::EntityPath;
use re_log_types::external::arrow;
use re_sdk_types::Archetype as _;
use re_sdk_types::ComponentSet;
use re_sdk_types::archetypes;
use re_sdk_types::components;
use re_viewer_context::{
    IdentifiedViewSystem, ViewContext, ViewContextCollection, ViewQuery, ViewSystemExecutionError,
    VisualizerExecutionOutput, VisualizerQueryInfo, VisualizerSystem,
};

// ---

#[derive(Debug, Clone)]
pub struct RawBytesEntry {
    pub path: EntityPath,
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

// TODO: this file is incredibly repetitive and has so much code duplication
// consider making the entire file a macro?

impl VisualizerSystem for RawBytesSystem {
    fn visualizer_query_info(
        &self,
        _app_options: &re_viewer_context::AppOptions,
    ) -> VisualizerQueryInfo {
        VisualizerQueryInfo {
            // works: gives all entities
            relevant_archetype: None,

            // // broken: only gives RawBytes
            // relevant_archetype: archetypes::RawBytes::name().into(),

            // // broken: only gives RawBytes blobs
            // constraints: SingleRequiredComponentConstraint::new::<components::Blob>(
            //     &archetypes::RawBytes::descriptor_blob(),
            // )
            // .with_additional_physical_types(SUPPORTED_DATATYPES.into_iter().cloned())
            // .with_allow_static_data(false)
            // .into(),

            // works: gives all entities
            constraints: re_viewer_context::VisualizabilityConstraints::AnyBuiltinComponent(
                itertools::chain!(
                    archetypes::RawBytes::all_component_identifiers(),
                    archetypes::TextDocument::all_component_identifiers(),
                )
                .collect::<ComponentSet>(),
            ),

            // TODO: I don't know what this does - it doesn't seem to affect anything
            queried: itertools::chain!(
                archetypes::RawBytes::all_components().into_owned(),
                archetypes::TextDocument::all_components().into_owned(),
            )
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

        let all_components = itertools::chain!(
            archetypes::RawBytes::all_component_identifiers(),
            archetypes::TextDocument::all_component_identifiers(),
        )
        .collect_vec();
        for (data_result, instruction) in
            view_query.iter_visualizer_instruction_for(Self::identifier())
        {
            let results = re_view::latest_at_with_blueprint_resolved_data(
                ctx,
                None,
                &view_query.latest_at_query(),
                data_result,
                all_components.iter().copied(),
                Some(instruction),
            );

            // TODO: should we use `results.get_unit_chunk()` instead of `.get_mono()`?
            // does that allow for multiple components of the same type per entity?
            // TODO: is there a better way to just iterate all the components
            // rather than writing them again here?

            // extract components one-by-one
            let bufs = [
                results
                    .get_mono::<components::Blob>(archetypes::RawBytes::descriptor_blob().component)
                    .map(|blob| blob.0.0.into_inner()),
                results
                    .get_mono::<components::Text>(
                        archetypes::TextDocument::descriptor_text().component,
                    )
                    .map(|text| text.0.0.into_arrow_buffer()),
            ];

            bufs.into_iter().flatten().for_each(|buf| {
                entries.push(RawBytesEntry {
                    buf,
                    path: results.entity_path().clone(),
                });
            });
        }

        Ok(VisualizerExecutionOutput::default().with_visualizer_data(entries))
    }
}
