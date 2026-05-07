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

macro_rules! implementation {
    ($(
        $component:path {
            $( $archetype:path > $field:ident ),+ $(,)?
        } => |$var:ident| $map:expr
    ),* $(,)?) => {
        /// The set of all `ComponentIdentifier`s that this visualiser is able to read from
        static COMPONENT_IDENTIFIERS
            : std::sync::LazyLock<re_sdk_types::ComponentSet>
            = std::sync::LazyLock::new(|| { paste::paste!{
                re_sdk_types::ComponentSet::from_iter([ $( $(
                    <$archetype>:: [< descriptor_ $field >] ().component,
                )+ )* ])
            } } );

        /// The set of all `ComponentDescriptor`s that this visualiser can read from.
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
                                .map(|$var : $component| $map)
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
        archetypes::Asset3D > blob,
        archetypes::AssetVideo > blob,
        archetypes::EncodedDepthImage > blob,
        archetypes::EncodedImage > blob,
        archetypes::McapMessage > data,
        archetypes::McapSchema > data,
        archetypes::RawBytes > blob,
    } => |blob| blob.0.0.into_inner(),
    components::Text {
        archetypes::TextDocument > text,
        archetypes::Status > status,
        archetypes::TextLog > text,
        archetypes::McapSchema > name,
        archetypes::McapSchema > encoding,
        archetypes::McapChannel > topic,
        archetypes::McapChannel > message_encoding,
    } => |text| text.0.0.into_arrow_buffer(),
    components::ImageBuffer {
        archetypes::Image > buffer,
        archetypes::Mesh3D > albedo_texture_buffer,
        archetypes::GridMap > data,
        archetypes::DepthImage > buffer,
        archetypes::SegmentationImage > buffer,
    } => |img| img.0.0.into_inner(),
    components::VideoSample {
        archetypes::VideoStream > sample,
    } => |sample| sample.0.0.into_inner(),

    // TODO: these are arrays not elements, and are currently unsupported

    // [components::Text] {
    //     archetypes::Boxes2D > labels,
    //     archetypes::Boxes3D > labels,
    //     archetypes::Points2D > labels,
    //     archetypes::Points3D > labels,
    //     archetypes::Arrows2D > labels,
    //     archetypes::Arrows3D > labels,
    //     archetypes::LineStrips2D > labels,
    //     archetypes::LineStrips3D > labels,
    //     archetypes::Capsules3D > labels,
    //     archetypes::Cylinders3D > labels,
    //     archetypes::GraphNodes > labels,
    // } => |text| text.0.0.into_arrow_buffer(),
    // [components::Scalar] {
    //     archetypes::Scalars > scalars,
    //     // these two aren't yet implemented, but might be in the future
    //     // (see comments in their codegen `.fbs`)
    //     // archetypes::SeriesLines > scalars,
    //     // archetypes::SeriesPoints > scalars,
    // } => |scalar| todo!("maybe use `bytemuck` to cast"),
}
