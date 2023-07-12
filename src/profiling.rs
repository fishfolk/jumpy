use tracing_core::Field;
use tracing_subscriber::{field::MakeVisitor, fmt::format::Writer};

/// Format scope such that field 'name' is displayed as
/// its value instead of field debug formatting (like name="{value}")
#[derive(Default, Debug)]
pub struct ProfilingScopeFormatter;

/// Visitor implementing formatting for [`ProfilingScopeFormatter`]
pub struct ProfilingScopeVisitor<'a> {
    writer: Writer<'a>,
    is_empty: bool,
    result: Result<(), std::fmt::Error>,
}

impl<'a> ProfilingScopeVisitor<'a> {
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

impl<'a> tracing_subscriber::field::Visit for ProfilingScopeVisitor<'a> {
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

impl<'a> tracing_subscriber::field::VisitOutput<std::fmt::Result> for ProfilingScopeVisitor<'a> {
    fn finish(self) -> std::fmt::Result {
        self.result
    }
}

impl<'a> tracing_subscriber::field::VisitFmt for ProfilingScopeVisitor<'a> {
    fn writer(&mut self) -> &mut dyn std::fmt::Write {
        &mut self.writer
    }
}

impl<'a> MakeVisitor<Writer<'a>> for ProfilingScopeFormatter {
    type Visitor = ProfilingScopeVisitor<'a>;

    #[inline]
    fn make_visitor(&self, target: Writer<'a>) -> Self::Visitor {
        ProfilingScopeVisitor::new(target, true)
    }
}

/// Notify profilers we are at frame boundary
pub fn mark_new_frame() {
    // Tell puffin we are on new frame
    puffin::GlobalProfiler::lock().new_frame();

    #[cfg(feature = "profiling-full")]
    // tell tracy we are on new frame
    tracing::event!(
        tracing::Level::INFO,
        message = "finished frame",
        tracy.frame_mark = true
    );
}
