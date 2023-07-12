///! This is based the work done here: https://github.com/bevyengine/bevy/pull/4730
///! This creates puffin scopes from tracing - integrating bevy's profiling traces with puffin.
use puffin::ThreadProfiler;
use std::{cell::RefCell, collections::VecDeque};
use tracing_core::{
    span::{Attributes, Id, Record},
    Subscriber,
};
use tracing_subscriber::{
    fmt::{format::DefaultFields, FormatFields, FormattedFields},
    layer::Context,
    registry::LookupSpan,
    Layer,
};

thread_local! {
    static PUFFIN_SPAN_STACK: RefCell<VecDeque<(Id, usize)>> =
        RefCell::new(VecDeque::with_capacity(16));
}

/// A tracing layer that collects data for puffin.
pub struct PuffinLayer<F = DefaultFields> {
    fmt: F,
}

impl Default for PuffinLayer<DefaultFields> {
    fn default() -> Self {
        Self {
            fmt: DefaultFields::default(),
        }
    }
}

impl PuffinLayer<DefaultFields> {
    /// Create a new `PuffinLayer`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Use a custom field formatting implementation.
    pub fn with_formatter<F>(self, fmt: F) -> PuffinLayer<F> {
        let _ = self;
        PuffinLayer { fmt }
    }
}

impl<S, F> Layer<S> for PuffinLayer<F>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    F: for<'writer> FormatFields<'writer> + 'static,
{
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        if !puffin::are_scopes_on() {
            return;
        }

        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            if extensions.get_mut::<FormattedFields<F>>().is_none() {
                let mut fields = FormattedFields::<F>::new(String::with_capacity(64));
                if self.fmt.format_fields(fields.as_writer(), attrs).is_ok() {
                    extensions.insert(fields);
                }
            }
        }
    }

    fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            if let Some(fields) = extensions.get_mut::<FormattedFields<F>>() {
                let _ = self.fmt.add_fields(fields, values);
            } else {
                let mut fields = FormattedFields::<F>::new(String::with_capacity(64));
                if self.fmt.format_fields(fields.as_writer(), values).is_ok() {
                    extensions.insert(fields);
                }
            }
        }
    }

    fn on_enter(&self, id: &Id, ctx: Context<'_, S>) {
        if !puffin::are_scopes_on() {
            return;
        }

        if let Some(span_data) = ctx.span(id) {
            let metadata = span_data.metadata();
            let name = metadata.name();
            let target = metadata.target();
            let extensions = span_data.extensions();
            let data = extensions
                .get::<FormattedFields<F>>()
                .map(|fields| fields.fields.as_str())
                .unwrap_or_default();

            ThreadProfiler::call(|tp| {
                let start_stream_offset = tp.begin_scope(name, target, data);
                PUFFIN_SPAN_STACK.with(|s| {
                    s.borrow_mut().push_back((id.clone(), start_stream_offset));
                });
            });
        }
    }

    fn on_exit(&self, id: &Id, _ctx: Context<'_, S>) {
        PUFFIN_SPAN_STACK.with(|s| {
            let value = s.borrow_mut().pop_back();
            if let Some((last_id, start_stream_offset)) = value {
                if *id == last_id {
                    ThreadProfiler::call(|tp| tp.end_scope(start_stream_offset));
                } else {
                    s.borrow_mut().push_back((last_id, start_stream_offset));
                }
            }
        });
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            span.extensions_mut().remove::<FormattedFields<F>>();
        }
    }
}
