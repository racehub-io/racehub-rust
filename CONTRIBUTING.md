# Contributing

Contributions can improve the local collector, file formats, tests, or documentation.
Read the [main README](README.md) and [capture developer guide](capture/README.md) first.

## Report a problem

Open a GitHub issue with:

- The observed result and the expected result.
- The game, packet format, bridge version, and operating system.
- The RaceHub source revision and Rust version.
- The recording scope, relevant command, and steps to reproduce.
- The final counters and error text, with private data removed.

Do not post credentials, account data, private recordings, or sensitive packet dumps.
Use [SECURITY.md](SECURITY.md) for a private security report route.
Ask for a private data-sharing route before you supply a recording that is not public.

## Prepare a change

Keep each pull request focused on one problem.
Explain the new behavior and the checks that support it.
Use short, direct English in code comments and documentation.
Keep existing contributor names and attribution.

For telemetry changes:

- Preserve source units and 64-bit session IDs.
- Keep different games and source sessions separate.
- Keep absent channels absent.
- Preserve packet times and the source times of carried values.
- Keep memory and input sizes bounded.
- Keep normal stop, incomplete files, and recovery behavior explicit.
- Update the relevant command or format documentation when its behavior changes.

Use actual recordings for positive tests.
Only add data that you have permission to publish.
Include its origin and licence beside it.
Do not invent telemetry measurements or change source packet bytes to make a test pass.
Empty, oversized, and malformed inputs are suitable for negative tests.
Do not include game SDK files or proprietary binary dependencies without their required rights.

## Run checks

From the repository root:

```sh
cargo build --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked
```

Format the Rust files that you change.
Avoid unrelated formatting changes to the legacy source.
If you change the Python converter, run its focused error check with Ruff:

```sh
ruff check --select F converter.py
```

State which checks passed and which checks you could not run.
Keep parser tests, repeated-packet load tests, and live game tests distinct.
A decoder test is not proof of live Windows capture or complete-lap accuracy.
When you report performance, give the build profile, input origin, repetition count,
operating system, and measurement scope.
Do not present a parser benchmark as network or disk throughput.

## Contribution terms

Read [LICENSING.md](LICENSING.md) and the full [CC0 1.0 terms](LICENSES/CC0-1.0.txt).
State that you offer new original contributions under CC0 1.0.
Only grant rights that you own.
Keep existing terms for material from other contributors and vendors.

Preserve the protocol [NOTICE](capture/src/protocols/NOTICE),
the [rFactor 2 MIT notice](capture/src/protocols/LICENSE.rfactor2),
and the [recording MIT licence](capture/tests/recordings/LICENSE).
Record the origin and licence of any copied or generated material.
The licence scope lists source origins that still need review.
Do not describe all repository content as public domain.

Use respectful, direct language.
Maintenance depends on volunteer time.
No response time or release date is promised.
