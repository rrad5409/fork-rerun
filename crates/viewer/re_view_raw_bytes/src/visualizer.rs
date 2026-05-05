use re_log_types::EntityPath;
use re_log_types::external::arrow;
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
    pub component: re_sdk_types::ComponentIdentifier,
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
macro_rules! implementation {
    ($(
        $component:path {
            $( $archetype:path > $field:ident ),+ $(,)?
        } => |$var:ident| $map:expr
    ),* $(,)?) => {
        // TODO: make this `const` once `Archetype::descriptor_xx()` becomes `const fn`
        static COMPONENT_IDENTIFIERS
            : std::sync::LazyLock<re_sdk_types::ComponentSet>
            = std::sync::LazyLock::new(|| { paste::paste!{
                re_sdk_types::ComponentSet::from_iter([ $( $(
                    <$archetype>:: [< descriptor_ $field >] ().component,
                )+ )* ])
            } } );
        static COMPONENT_DESCRIPTORS
            : std::sync::LazyLock<re_viewer_context::SortedComponentSet>
            = std::sync::LazyLock::new(|| { paste::paste!{
                std::iter::FromIterator::from_iter([ $( $(
                    <$archetype>:: [< descriptor_ $field >] (),
                )+ )* ])
            } } );

        impl VisualizerSystem for RawBytesSystem {
            fn visualizer_query_info(
                &self,
                _app_options: &re_viewer_context::AppOptions,
            ) -> VisualizerQueryInfo {
                VisualizerQueryInfo {
                    relevant_archetype: None,
                    constraints: re_viewer_context::VisualizabilityConstraints::AnyBuiltinComponent(
                        COMPONENT_IDENTIFIERS.clone()
                    ),

                    // TODO: I don't know what this does - it doesn't seem to affect anything
                    queried: COMPONENT_DESCRIPTORS.clone(),
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
                        COMPONENT_IDENTIFIERS.iter().copied(),
                        Some(instruction),
                    );

                    // TODO: should we use `results.get_unit_chunk()` instead of `.get_mono()`?
                    // does that allow for multiple components of the same type per entity?

                    // extract components one-by-one
                    let bufs = paste::paste!{ [
                        $( $(
                            results
                                .get_mono::<$component>(
                                    <$archetype>:: [< descriptor_ $field >] ().component
                                )
                                .map(|$var| $map)
                                .map(|buf| (buf, <$archetype>:: [< descriptor_ $field >] ().component)),
                        )+ )*
                    ] };

                    bufs.into_iter().flatten().for_each(|(buf, component)| {
                        entries.push(RawBytesEntry {
                            buf,
                            component,
                            path: results.entity_path().clone(),
                        });
                    });
                }

                Ok(VisualizerExecutionOutput::default().with_visualizer_data(entries))
            }
        }
    };
}

implementation! {
    components::Blob {
        archetypes::RawBytes > blob,
    } => |blob| blob.0.0.into_inner(),
    components::Text {
        archetypes::TextDocument > text,
    } => |text| text.0.0.into_arrow_buffer()
}
