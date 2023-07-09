///! This is based the work done here: https://github.com/bevyengine/bevy/pull/4730
///! This creates puffin scopes from tracing - integrating bevy's profiling traces with puffin.
use puffin::ThreadProfiler;
use std::{cell::RefCell, collections::VecDeque};
use tracing_core::{
    span::{Attributes, Id, Record},
    Field, Subscriber,
};
use tracing_subscriber::{
    field::MakeVisitor,
    fmt::{
        format::{DefaultFields, Writer},
        FormatFields, FormattedFields,
    },
    layer::Context,
    registry::LookupSpan,
    Layer,
};

/// Format puffin scope such that field 'name' is displayed as
/// its value instead of field debug formatting (like name="{value}")
#[derive(Default, Debug)]
pub struct PuffinScopeFormatter;

/// Visitor implementing formatting for [`PuffinScopeFormatter`]
pub struct PuffinScopeVisitor<'a> {
    writer: Writer<'a>,
    is_empty: bool,
    result: Result<(), std::fmt::Error>,
}

impl<'a> PuffinScopeVisitor<'a> {
    pub fn new(writer: Writer<'a>, is_empty: bool) -> Self {
        Self {
            writer,
            is_empty,
            result: Ok(()),
        }
    }

    fn maybe_pad(&mut self) {
        if self.is_empty {
            self.is_empty = false;
        } else {
            self.result = write!(self.writer, " ");
        }
    }
}

impl<'a> tracing_subscriber::field::Visit for PuffinScopeVisitor<'a> {
    fn record_str(&mut self, field: &Field, value: &str) {
        if self.result.is_err() {
            return;
        }

        if field.name() == "name" {
            self.record_debug(field, &format_args!("{value}"))
        } else {
            // If fields other than 'name' included in span, debug print.
            self.record_debug(field, &value)
        }
    }

    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        if self.result.is_err() {
            return;
        }
        self.maybe_pad();
        self.result = write!(self.writer, "{value:?}");
    }
}

impl<'a> tracing_subscriber::field::VisitOutput<std::fmt::Result> for PuffinScopeVisitor<'a> {
    fn finish(self) -> std::fmt::Result {
        self.result
    }
}

impl<'a> tracing_subscriber::field::VisitFmt for PuffinScopeVisitor<'a> {
    fn writer(&mut self) -> &mut dyn std::fmt::Write {
        &mut self.writer
    }
}

impl<'a> MakeVisitor<Writer<'a>> for PuffinScopeFormatter {
    type Visitor = PuffinScopeVisitor<'a>;

    #[inline]
    fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor {
        PuffinScopeVisitor::new(target, true)
    }
}

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
