//! The logging implementation is duplicated from Bevy - `bevy::LogPlugin` does not support registering
//! external tracing Layers, this implementation adds an extra layer for sending tracing to the in
//! game console. If this feature is added in the future, we should switch back to Bevy's LogPlugin.

// #[cfg(target_os = "android")]
// mod android_tracing;

pub mod prelude {
    //! The Bevy Log Prelude.
    #[doc(hidden)]
    pub use bevy::utils::tracing::{
        debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    };
}

use bevy::prelude::{App, Plugin};
pub use bevy::utils::tracing::{
    debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    Level,
};

use tracing_log::LogTracer;
// #[cfg(feature = "tracing-chrome")]
// use tracing_subscriber::fmt::{format::DefaultFields, FormattedFields};
use tracing_subscriber::{prelude::*, registry::Registry, EnvFilter};

#[cfg(not(target_arch = "wasm32"))]
use crate::prelude::{ConsoleBufferResource, ConsoleLogBuffer, ConsoleLogBufferWriter};

/// This is largely duplicate of `bevy::LogPlugin` with minor additions. Some functionality
/// normally behind bevy_log feature flags is not yet implemented.
///
/// This plugin should be loaded as early as possible to ensure console_error_panic_hook
/// is set before any exceptions are thrown on wasm target.
///
/// Enabling [`JumpyLogPlugin`] requires disabling `bevy::LogPlugin`:
/// ```
///     App::new()
///         .add_plugins(DefaultPlugins.build().disable::<LogPlugin>())
///         .add_plugin(JumpyLogPlugin {
///             level: Level::DEBUG,
///             filter: "wgpu=error,bevy_render=info,bevy_ecs=trace".to_string(),
///         })
///         .run();
/// ```
///
/// Log level can also be changed using the `RUST_LOG` environment variable.
/// For example, using `RUST_LOG=wgpu=error,bevy_render=info,bevy_ecs=trace cargo run ..`
///
/// It has the same syntax as the field [`JumpyLogPlugin::filter`], see [`EnvFilter`].
/// If you define the `RUST_LOG` environment variable, the [`JumpyLogPlugin`] settings
/// will be ignored.
///
/// # Panics
///
/// This plugin should not be added multiple times in the same process. This plugin
/// sets up global logging configuration for **all** Apps in a given process, and
/// rerunning the same initialization multiple times will lead to a panic.
/// (See example of disabling `bevy::LogPlugin` from defaults in example above)
pub struct JumpyLogPlugin {
    /// Filters logs using the [`EnvFilter`] format
    pub filter: String,

    /// Filters out logs that are "less than" the given level.
    /// This can be further filtered using the `filter` setting.
    pub level: Level,
}

impl Default for JumpyLogPlugin {
    fn default() -> Self {
        Self {
            filter: "wgpu=error".to_string(),
            level: Level::INFO,
        }
    }
}

impl Plugin for JumpyLogPlugin {
    #[cfg_attr(not(feature = "tracing-chrome"), allow(unused_variables))]
    fn build(&self, app: &mut App) {
        // #[cfg(feature = "trace")]
        // {
        //     let old_handler = panic::take_hook();
        //     panic::set_hook(Box::new(move |infos| {
        //         println!("{}", tracing_error::SpanTrace::capture());
        //         old_handler(infos);
        //     }));
        // }

        let finished_subscriber;
        let default_filter = { format!("{},{}", self.level, self.filter) };
        let filter_layer = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&default_filter))
            .unwrap();

        let subscriber = Registry::default().with(filter_layer);

        // #[cfg(feature = "trace")]
        // let subscriber = subscriber.with(tracing_error::ErrorLayer::default());

        #[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
        {
            // Support for saving a chrome tracing file
            // #[cfg(feature = "tracing-chrome")]
            // let chrome_layer = {
            //     let mut layer = tracing_chrome::ChromeLayerBuilder::new();
            //     if let Ok(path) = std::env::var("TRACE_CHROME") {
            //         layer = layer.file(path);
            //     }
            //     let (chrome_layer, guard) = layer
            //         .name_fn(Box::new(|event_or_span| match event_or_span {
            //             tracing_chrome::EventOrSpan::Event(event) => event.metadata().name().into(),
            //             tracing_chrome::EventOrSpan::Span(span) => {
            //                 if let Some(fields) =
            //                     span.extensions().get::<FormattedFields<DefaultFields>>()
            //                 {
            //                     format!("{}: {}", span.metadata().name(), fields.fields.as_str())
            //                 } else {
            //                     span.metadata().name().into()
            //                 }
            //             }
            //         }))
            //         .build();
            //     app.world.insert_non_send_resource(guard);
            //     chrome_layer
            // };

            // #[cfg(feature = "tracing-tracy")]
            // let tracy_layer = tracing_tracy::TracyLayer::new();

            let fmt_layer = tracing_subscriber::fmt::Layer::default().with_writer(std::io::stderr);

            // bevy_render::renderer logs a `tracy.frame_mark` event every frame
            // at Level::INFO. Formatted logs should omit it.
            // #[cfg(feature = "tracing-tracy")]
            // let fmt_layer =
            //     fmt_layer.with_filter(tracing_subscriber::filter::FilterFn::new(|meta| {
            //         meta.fields().field("tracy.frame_mark").is_none()
            //     }));

            // Enable resource for console system
            let shared_buffer = ConsoleLogBuffer::default();
            let console_buffer_resource = ConsoleBufferResource::new(shared_buffer.clone());
            app.insert_resource(console_buffer_resource);

            // Layer to write logs for access in in-game console
            let console_writer = ConsoleLogBufferWriter::new(shared_buffer);
            let console_layer = tracing_subscriber::fmt::Layer::default()
                .with_ansi(false) // console does not support this
                .with_writer(console_writer);

            let subscriber = subscriber.with(fmt_layer).with(console_layer);

            // #[cfg(feature = "tracing-chrome")]
            // let subscriber = subscriber.with(chrome_layer);
            // #[cfg(feature = "tracing-tracy")]
            // let subscriber = subscriber.with(tracy_layer);

            finished_subscriber = subscriber;
        }

        #[cfg(target_arch = "wasm32")]
        {
            console_error_panic_hook::set_once();
            finished_subscriber = subscriber.with(tracing_wasm::WASMLayer::new(
                tracing_wasm::WASMLayerConfig::default(),
            ));
        }

        // #[cfg(target_os = "android")]
        // {
        //     finished_subscriber = subscriber.with(android_tracing::AndroidLayer::default());
        // }

        let logger_already_set = LogTracer::init().is_err();
        let subscriber_already_set =
            bevy::utils::tracing::subscriber::set_global_default(finished_subscriber).is_err();

        match (logger_already_set, subscriber_already_set) {
			(true, true) => warn!(
				"Could not set global logger and tracing subscriber as they are already set. Consider disabling JumpyLogPlugin."
			),
			(true, _) => warn!("Could not set global logger as it is already set. Consider disabling JumpyLogPlugin."),
			(_, true) => warn!("Could not set global tracing subscriber as it is already set. Consider disabling JumpyLogPlugin."),
			_ => (),
		}
    }
}
