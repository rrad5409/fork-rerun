use re_log_types::EntityPath;
use re_log_types::external::arrow;
use re_sdk_types::Archetype as _;
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

static REFLECTION: std::sync::LazyLock<re_sdk_types::reflection::Reflection> =
    std::sync::LazyLock::new(|| {
        re_sdk_types::reflection::generate_reflection().expect("failed to generate reflection data")
    });

/// All the datatypes that this visualizer can process
///
/// This should be every registered type from all components
static DATATYPES: std::sync::LazyLock<std::collections::HashSet<arrow::datatypes::DataType>> =
    std::sync::LazyLock::new(|| {
        REFLECTION
            .components
            .values()
            .cloned()
            .map(|val| val.datatype)
            .collect()
    });

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
            .with_additional_physical_types(DATATYPES.iter().cloned())
            .into(),
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
                    path: results.entity_path().clone(),
                });
            };
        }

        Ok(VisualizerExecutionOutput::default().with_visualizer_data(entries))
    }
}
