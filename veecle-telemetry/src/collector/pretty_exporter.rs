use super::Export;
use crate::protocol::{InstanceMessage, LogMessage, TelemetryMessage};
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
/// use veecle_telemetry::collector::{ConsolePrettyExporter, set_exporter};
/// use veecle_telemetry::protocol::ExecutionId;
///
/// let execution_id = ExecutionId::random(&mut rand::rng());
/// set_exporter(execution_id, &ConsolePrettyExporter::DEFAULT).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct ConsolePrettyExporter(());

impl ConsolePrettyExporter {
    /// A `const` version of `ConsolePrettyExporter::default()` to allow use as an `&'static`.
    pub const DEFAULT: Self = ConsolePrettyExporter(());
}

impl Export for ConsolePrettyExporter {
    fn export(
        &self,
        InstanceMessage {
            execution: _,
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
        let attributes = attributes
            .iter()
            .fold(String::new(), |mut formatted, key_value| {
                formatted.push_str(", ");
                formatted.push_str(&std::format!("{}", key_value));
                formatted
            });
        std::writeln!(
            output,
            "[{severity:?}:{time_unix_nano}] {body}: \"{attributes}\"",
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::format_message;
    use crate::macros::attributes;
    use crate::protocol::{LogMessage, Severity, TelemetryMessage};
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
                    span_id: None,
                    trace_id: None,
                    time_unix_nano,
                    severity,
                    body: body.into(),
                    attributes: attributes.into(),
                }),
                &mut output,
            );
        }

        assert_eq!(
            str::from_utf8(&output).unwrap(),
            indoc! { r#"
            [Trace:1000000] booting: ""
            [Debug:5000000] booted: ", truth: true, lies: false"
            [Info:5000000000] running: ", mille: 1000, milli: 0.001"
            [Warn:60000000000] running late: ""
            [Error:61000000000] really late: ""
            [Fatal:3600000000000] terminating: ""
            [Trace:2703621600000000000] Then are _we_ inhabited by history: ""
            [Debug:2821816800000000000] Light dawns and marble heads, what the hell does this mean: ""
            [Info:2860956000000000000] This terror that hunts: ", Typed: true, date: 1960-08-29"
            [Warn:3118950000000000000] I have no words, the finest cenotaph: ""
            [Error:3119036400000000000] A sun to read the dark: ", or: A son to rend the dark"
            [Fatal:3122146800000000000] _Tirer comme des lapins_: ", translated: Shot like rabbits"
        "# }
        );
    }
}
