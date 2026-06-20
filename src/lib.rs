use std::fmt::Write;

use godot::global::{PrintLevel, PrintRecord, PrintSource};
use tracing::{Level, Metadata};
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, util::SubscriberInitExt};

pub(super) fn initialize_tracing() {
    tracing_subscriber::registry().with(GodotLayer {}).init();
}

fn to_godot_level(level: &tracing::Level) -> PrintLevel {
    if *level <= Level::ERROR {
        PrintLevel::Error
    } else if *level <= Level::WARN {
        PrintLevel::Warn
    } else {
        PrintLevel::Info
    }
}

fn to_godot_source<'meta>(meta: &'meta Metadata) -> Option<PrintSource<'meta>> {
    Some(PrintSource {
        file: meta.file()?,
        function: meta.module_path().unwrap_or(""),
        line: meta.line().unwrap_or(0),
    })
}

struct GodotVisitor {
    message: Option<String>,
    fields: String,
}

impl GodotVisitor {
    fn new() -> Self {
        Self {
            message: None,
            fields: String::new(),
        }
    }
}

impl tracing::field::Visit for GodotVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
        if field.name() == "message" {
            let mut msg = String::new();
            let _ = write!(msg, "{:?}", value);
            self.message = Some(msg);
        } else {
            let _ = write!(self.fields, "{} = {:?}, ", field.name(), value);
        }
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = Some(value.to_owned());
        } else {
            let _ = write!(self.fields, "{} = {:?}, ", field.name(), value);
        }
    }
}

pub struct GodotLayer {}

impl<S> tracing_subscriber::Layer<S> for GodotLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
        let meta = event.metadata();
        if meta.is_span() {
            return;
        }

        let mut visitor = GodotVisitor::new();
        event.record(&mut visitor);

        let message = if let Some(mod_path) = meta.module_path()
            && meta.target() == mod_path
        {
            visitor.message.unwrap_or_default()
        } else {
            if let Some(msg) = visitor.message {
                format!("[{}] {msg}", meta.target())
            } else {
                format!("[{}]", meta.target())
            }
        };

        godot::global::print_custom(PrintRecord {
            level: to_godot_level(meta.level()),
            message: &message,
            rationale: if visitor.fields.is_empty() {
                None
            } else {
                Some(&visitor.fields)
            },
            source: to_godot_source(meta),
            editor_notify: false,
        });
    }
}
