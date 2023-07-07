use std::{
    io,
    sync::{Arc, Mutex},
};

#[allow(unused_imports)]
use bevy::prelude::{App, EventWriter, IntoSystemConfig, Plugin, Res, Resource};
use tracing_subscriber::fmt::MakeWriter;

#[cfg(not(target_arch = "wasm32"))]
use bevy_console::{ConsolePlugin, ConsoleSet, PrintConsoleLine};

/// A shared buffer for exposing logs to bevy systems,
/// primarily used to display logs in in-game console.
#[derive(Clone)]
pub struct ConsoleLogBuffer(pub Arc<Mutex<Vec<u8>>>);

impl Default for ConsoleLogBuffer {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(vec![])))
    }
}

/// Used by `tracing_subscriber::fmt::Layer` to write to `SharedLogBuffer`.
#[derive(Clone)]
pub struct ConsoleLogBufferWriter(ConsoleLogBuffer);

impl ConsoleLogBufferWriter {
    pub fn new(buffer: ConsoleLogBuffer) -> Self {
        Self(buffer)
    }
}

impl<'a> MakeWriter<'a> for ConsoleLogBufferWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        // Unfortunately this is invoked on each write in tracing_subscriber::fmt::Layer,
        // we clone our shared buffer (cloning an Arc internally). ideally we would
        // return Writer with direct reference to buffer internal to Arc, however a
        // self-referential struct is difficult in Rust.
        //
        // TODO: Explore storing ref to internal buffer alongside Arc,
        // and return ref here to avoid cost of cloning Arc on each write.
        self.clone()
    }
}

impl std::io::Write for ConsoleLogBufferWriter {
    fn write(&mut self, val: &[u8]) -> io::Result<usize> {
        let mut buffer_write = self.0 .0.lock().unwrap();
        let bytes = val.len();
        buffer_write.extend_from_slice(val);
        Ok(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        // Not needed
        Ok(())
    }
}

/// Resource exposing shared byte buffer to `print_console_log` system.
/// Internal buffer is writen to by tracing in `JumpyLogPlugin`.
///
/// (It may be written to by other systems for injecting console messages as well,
///  though really tracing should be used unless there is a good console specific reason)
#[derive(Resource, Clone)]
pub struct ConsoleBufferResource(ConsoleLogBuffer);

impl ConsoleBufferResource {
    pub fn new(buffer: ConsoleLogBuffer) -> Self {
        Self(buffer)
    }
}

/// System consuming messages from shared buffer and sending to console.
#[cfg(not(target_arch = "wasm32"))]
fn print_console_logs(
    buffer: Res<ConsoleBufferResource>,
    mut writer: EventWriter<PrintConsoleLine>,
) {
    let console_line;

    // WARNING: Do not put any tracing in this block, it will deadlock.
    // (The tracing writer will attempt to acquire this Mutex and block)
    {
        // Acquire lock, convert message to string and clear buffer
        let mut buffer_write = buffer.0 .0.lock().unwrap();
        if buffer_write.is_empty() {
            return;
        }

        // TODO: Consider changing to a double buffer or something.
        // This would only block writes from tracing for duration of buffer swap.
        // Current impl is naive and blocks tracing longer which could impact
        // perf on main thread.

        console_line = std::str::from_utf8(buffer_write.as_slice())
            .unwrap()
            .to_string();
        buffer_write.clear();
    }

    writer.send(PrintConsoleLine {
        line: console_line.into(),
    });
}

/// Plugin enabling a development console, activated by default with the 'grave' key: `
/// `ConsoleConfiguration` resource may be specified to change settings and regiser consoole commands.
pub struct JumpyConsolePlugin;

impl Plugin for JumpyConsolePlugin {
    #[allow(unused_variables)]
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugin(ConsolePlugin)
            // This resource may optionally be added by User to register commands or change settings.
            //
            //  .insert_resource(ConsoleConfiguration {
            //      ..Default::default()
            // })
            .add_system(print_console_logs.after(ConsoleSet::ConsoleUI));
    }
}
