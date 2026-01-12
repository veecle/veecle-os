use super::Export;
use crate::protocol::transient::{InstanceMessage, LogMessage, TelemetryMessage};
use std::string::String;

/// Exporter that pretty prints telemetry messages to stderr.
///
/// This exporter only supports log messages (e.g. `error!("foo")`).
///
/// <div class="warning">
/// Only intended for experimentation and examples.
/// `telemetry-ui` is strongly recommended for anything beyond experimentation.
/// </div>
///
/// # Examples
///
/// ```rust
/// use veecle_osal_std::{time::Time, thread::Thread};
/// use veecle_telemetry::collector::ConsolePrettyExporter;
///
/// veecle_telemetry::collector::build()
///     .random_process_id()
///     .exporter(&ConsolePrettyExporter::DEFAULT)
///     .time::<Time>()
///     .thread::<Thread>()
///     .set_global()
///     .unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ConsolePrettyExporter(());

impl ConsolePrettyExporter {
    /// A `const` version of `ConsolePrettyExporter::default()` to allow use as a `&'static`.
    pub const DEFAULT: Self = ConsolePrettyExporter(());
}

impl Export for ConsolePrettyExporter {
    fn export(
        &self,
        InstanceMessage {
            thread_id: _,
            message,
        }: InstanceMessage,
    ) {
        format_message(message, std::io::stderr());
    }
}

fn format_message(message: TelemetryMessage, mut output: impl std::io::Write) {
    if let TelemetryMessage::Log(LogMessage {
        time_unix_nano,
        severity,
        body,
        attributes,
        ..
    }) = message
    {
        // Millisecond accuracy is probably enough for a console logger.
        let time = time_unix_nano / 1_000_000;

        let attributes = if attributes.is_empty() {
            String::new()
        } else {
            let mut attributes =
                attributes
                    .iter()
                    .fold(String::from(" ["), |mut formatted, key_value| {
                        use std::fmt::Write;
                        write!(formatted, "{key_value}, ").unwrap();
                        formatted
                    });
            // Remove trailing `, `.
            attributes.truncate(attributes.len() - 2);
            attributes + "]"
        };

        // `Debug` doesn't apply padding, so pre-render to allow padding below.
        let severity = std::format!("{severity:?}");

        // Severity is up to 5 characters, pad it to stay consistent.
        //
        // Using a min-width of 6 for time means that if it is boot-time it will remain
        // consistently 6 digits wide until ~15 minutes have passed, after that it changes
        // slowly enough to not be distracting.
        // For Unix time it will already be 13 digits wide until 2286.
        std::writeln!(output, "[{severity:>5}:{time:6}] {body}{attributes}").unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::format_message;
    use crate::attributes;
    use crate::protocol::transient::{LogMessage, Severity, TelemetryMessage};
    use indoc::indoc;
    use pretty_assertions::assert_eq;
    use std::vec::Vec;

    #[test]
    fn smoke_test() {
        let mut output = Vec::new();

        let ns = 1_000_000_000;
        let messages = [
            // First some "boot time" messages with very low timestamps.
            (1_000_000, Severity::Trace, "booting", attributes!() as &[_]),
            (
                5_000_000,
                Severity::Debug,
                "booted",
                attributes!(truth = true, lies = false),
            ),
            (
                5 * ns,
                Severity::Info,
                "running",
                attributes!(mille = 1000, milli = 0.001),
            ),
            (60 * ns, Severity::Warn, "running late", attributes!()),
            (61 * ns, Severity::Error, "really late", attributes!()),
            (3600 * ns, Severity::Fatal, "terminating", attributes!()),
            // Then some "Unix time" messages sent around 2060.
            (
                2703621600 * ns,
                Severity::Trace,
                "Then are _we_ inhabited by history",
                attributes!() as &[_],
            ),
            (
                2821816800 * ns,
                Severity::Debug,
                "Light dawns and marble heads, what the hell does this mean",
                attributes!(),
            ),
            (
                2860956000 * ns,
                Severity::Info,
                "This terror that hunts",
                attributes!(Typed = true, date = "1960-08-29"),
            ),
            (
                3118950000 * ns,
                Severity::Warn,
                "I have no words, the finest cenotaph",
                attributes!(),
            ),
            (
                3119036400 * ns,
                Severity::Error,
                "A sun to read the dark",
                attributes!(or = "A son to rend the dark"),
            ),
            (
                3122146800 * ns,
                Severity::Fatal,
                "_Tirer comme des lapins_",
                attributes!(translated = "Shot like rabbits"),
            ),
        ];

        for (time_unix_nano, severity, body, attributes) in messages {
            format_message(
                TelemetryMessage::Log(LogMessage {
                    time_unix_nano,
                    severity,
                    body,
                    attributes,
                }),
                &mut output,
            );
        }

        assert_eq!(
            str::from_utf8(&output).unwrap(),
            indoc! { r#"
            [Trace:     1] booting
            [Debug:     5] booted [truth: true, lies: false]
            [ Info:  5000] running [mille: 1000, milli: 0.001]
            [ Warn: 60000] running late
            [Error: 61000] really late
            [Fatal:3600000] terminating
            [Trace:2703621600000] Then are _we_ inhabited by history
            [Debug:2821816800000] Light dawns and marble heads, what the hell does this mean
            [ Info:2860956000000] This terror that hunts [Typed: true, date: "1960-08-29"]
            [ Warn:3118950000000] I have no words, the finest cenotaph
            [Error:3119036400000] A sun to read the dark [or: "A son to rend the dark"]
            [Fatal:3122146800000] _Tirer comme des lapins_ [translated: "Shot like rabbits"]
        "# }
        );
    }
}
