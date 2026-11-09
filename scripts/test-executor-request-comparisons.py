#!/usr/bin/env python3
"""Compare the adapter's five request entries without a node.

The default mode checks the live readings against COMPARISONS byte for byte.
The --emit mode prints those readings for deliberate fixture re-admission.
Both modes execute the adapter source bytes they hash, without a bytecode cache.
"""

import argparse
import hashlib
import io
from pathlib import Path
import platform
import sys
import types


SOURCE_SHA256 = "e60999d8427b129b6caf82e2fefae553ec6ceb321327de726f08fc831bedc596"
MAXIMUM = 4194304


def encoded(value):
    """Encode text with its UTF-8 byte length and lowercase hexadecimal."""
    data = value.encode("utf-8")
    return f"{len(data)} {data.hex()}"


def digest(data):
    """Bind one exact operand."""
    return hashlib.sha256(data).hexdigest()


def readings(root):
    """Run every control and changed subject through its own entry."""
    path = root / "scripts/elements-native-executor.py"
    source = path.read_bytes()
    source_digest = digest(source)
    if source_digest != SOURCE_SHA256:
        raise ValueError("adapter source digest differs; deliberate re-admission required")
    adapter = types.ModuleType("elements_native_executor_request_comparisons")
    adapter.__file__ = str(path)
    exec(compile(source, str(path), "exec"), adapter.__dict__)
    if (adapter.COMMAND_NAME, adapter.ADAPTER_VERSION, adapter.NATIVE_PROTOCOL_SCHEMA,
            adapter.MAXIMUM_REQUEST_BYTES) != ("elements-native-executor", "2.1.1", 8, MAXIMUM):
        raise ValueError("adapter identity or request bound differs")
    pairs = (
        ("blank-request", "RequestFraming", "RequestReader.read", b"{}\n", b"  \n",
         adapter.FRAMING_BLANK_RECORD),
        ("oversized-request", "RequestFraming", "RequestReader.read", b"{}\n",
         b"{}" + b" " * (MAXIMUM - 1) + b"\n", adapter.FRAMING_OVERSIZED_RECORD),
        ("unterminated-request", "RequestFraming", "RequestReader.read", b"{}\n", b"{}",
         adapter.FRAMING_UNTERMINATED_RECORD),
        ("malformed-json", "RequestJson", "RequestReader.read_object", b"{}\n", b"{]\n",
         adapter.FRAMING_MALFORMED_RECORD),
        ("unknown-request-field", "RequestFields", "require_execution_request_fields",
         b'{"schema":8,"case":{},"subject":{}}\n',
         b'{"schema":8,"case":{},"subject":{},"unknown":0}\n',
         "EXECUTION_REQUEST_FIELD_CENSUS_FAILED"),
    )
    lines = []
    for key, boundary, entry, control, subject, expected in pairs:
        def observe(data):
            reader = adapter.RequestReader(io.BytesIO(data))
            if boundary == "RequestFraming":
                return reader.read(MAXIMUM, "request")
            value = reader.read_object(MAXIMUM, "request")
            if boundary == "RequestFields":
                adapter.require_execution_request_fields(value, ("schema", "case", "subject"))
            return value

        accepted = observe(control)
        expected_control = "{}" if boundary == "RequestFraming" else (
            {"schema": 8, "case": {}, "subject": {}} if boundary == "RequestFields" else {})
        if accepted != expected_control:
            raise ValueError(f"{key}: control result differs")
        try:
            observe(subject)
        except adapter.ProtocolFramingError as error:
            if boundary == "RequestFields" or error.phase != "request":
                raise ValueError(f"{key}: wrong refusing boundary") from error
            refusal = error.failure
        except adapter.FatalAdapterError as error:
            if boundary != "RequestFields" or error.outcome is not (
                    adapter.DiagnosticOutcome.EXECUTION_REQUEST_FIELD_CENSUS_FAILED):
                raise ValueError(f"{key}: wrong refusing boundary") from error
            refusal = error.outcome.name
        else:
            raise ValueError(f"{key}: changed subject accepted")
        if refusal != expected:
            raise ValueError(f"{key}: refusal differs")
        lines.append(" ".join((
            "comparison", encoded("16.12 / " + key), boundary, encoded(entry),
            "accepted", "refused", encoded(refusal), str(len(control)), digest(control),
            str(len(subject)), digest(subject), encoded(adapter.COMMAND_NAME),
            encoded(adapter.ADAPTER_VERSION), str(adapter.NATIVE_PROTOCOL_SCHEMA),
            source_digest, encoded(platform.python_version()),
        )))
    return ("\n".join(lines) + "\n").encode("ascii")


def main():
    """Emit an actual run or compare it with the admitted five-line fixture."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--emit", action="store_true")
    parser.add_argument("--repository-root", type=Path,
                        default=Path(__file__).resolve().parent.parent)
    arguments = parser.parse_args()
    actual = readings(arguments.repository_root)
    if arguments.emit:
        sys.stdout.buffer.write(actual)
        return 0
    expected = (arguments.repository_root /
                "packages/vectors/fixtures/adapter-request-comparisons/COMPARISONS").read_bytes()
    if actual != expected:
        old, new = expected.splitlines(keepends=True), actual.splitlines(keepends=True)
        for index in range(max(len(old), len(new))):
            before = old[index] if index < len(old) else b"<missing>"
            after = new[index] if index < len(new) else b"<missing>"
            if before != after:
                print(f"comparison line {index + 1} differs: fixture={before!r}; live={after!r}",
                      file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
