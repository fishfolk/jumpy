use std::io;

use async_channel::{Receiver, Sender};
use bevy::prelude::default;
#[allow(unused_imports)]
use bevy::prelude::{App, EventWriter, IntoSystemConfig, Plugin, Res, Resource};
use byte_pool::BytePool;
use once_cell::sync::Lazy;

use bevy_console::{ConsolePlugin, ConsoleSet, PrintConsoleLine};

/// Sender and receiver for console log messages.
///
/// Messages should be allocated out of the [`CONSOLE_LOG_BUFFER].
pub struct ConsoleLogChannel {
    pub receiver: Receiver<byte_pool::Block<'static>>,
    pub sender: Sender<byte_pool::Block<'static>>,
}
impl Default for ConsoleLogChannel {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver }
    }
}

/// [`ConsoleLogChannel`] static for sending and receiving console log messages.
pub static CONSOLE_LOG_CHANNEL: Lazy<ConsoleLogChannel> = Lazy::new(default);
/// Static buffer pool for log messages to avoid re-allocating every time we send a message over the
/// channel.
pub static CONSOLE_LOG_BUFFER: Lazy<BytePool> = Lazy::new(default);

/// Used by `tracing_subscriber::fmt::Layer` to write to `SharedLogBuffer`.
#[derive(Default)]
pub struct ConsoleLogBufferWriter;

impl std::io::Write for ConsoleLogBufferWriter {
    fn write(&mut self, val: &[u8]) -> io::Result<usize> {
        let mut buf = CONSOLE_LOG_BUFFER.alloc(val.len());
        buf.copy_from_slice(val);
        CONSOLE_LOG_CHANNEL.sender.try_send(buf).unwrap();
        Ok(val.len())
    }

    // Not needed
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// System consuming messages from shared buffer and sending to console.
fn print_console_logs(mut writer: EventWriter<PrintConsoleLine>) {
    while let Ok(message) = CONSOLE_LOG_CHANNEL.receiver.try_recv() {
        let console_line = String::from_utf8(message.to_vec()).unwrap();

        writer.send(PrintConsoleLine {
            line: console_line.into(),
        });
    }
}

/// Plugin enabling a development console, activated by default with the 'grave' key: `
/// `ConsoleConfiguration` resource may be specified to change settings and regiser consoole commands.
pub struct JumpyConsolePlugin;

impl Plugin for JumpyConsolePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ConsolePlugin)
            // This resource may optionally be added by User to register commands or change settings.
            //
            //  .insert_resource(ConsoleConfiguration {
            //      ..Default::default()
            // })
            .add_system(print_console_logs.after(ConsoleSet::ConsoleUI));
    }
}
