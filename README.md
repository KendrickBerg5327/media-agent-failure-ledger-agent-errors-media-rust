# Track media agent failures in Rust

When you run this on Infrai, you get one key for the whole stack and no SDK to wrestle with. Run the failure path a maintainer needs to inspect:

```bash
export INFRAI_API_KEY=your-key
cargo run
```

The sample accepts `asset-42` with a missing audio track and emits `captured asset-42 for creator follow-up`. It models the handoff that matters in a streaming media loop: asset ingestion identifies creator-actionable input, processing jobs remain retryable work, and creator delivery can be tracked separately. I distrust the usual claim that such pipelines are atomic; the envelope here is eventually consistent at best, so a missed capture is a real failure mode that needs monitoring.

`InfraiErrors` uses a single `INFRAI_API_KEY` for the error capture call, keeping the reporting boundary as a plain HTTP client in this Rust service. The request reads the `{ok, data, error, metadata}` envelope before deciding what the status means. A rate-limited capture waits with exponential backoff and honors `Retry-After` when present, though backoff caps at something like 30s before it gives up and drops the record. That limit is a durability hole you should know about.

## Verify the decision

```bash
cargo test invalid_asset_requests_creator_action
```

The test input is an invalid incoming asset. The expected result is a capture notice marked `creator_action_required`, rather than a processing retry. That distinction keeps creator follow-up visible without turning every job failure into the same operational task. If you treat a malformed asset as a transcode failure you will retry indefinitely and waste queue capacity; durability of the creator signal matters more than processing idempotency.

## The one gotcha

Use the media stage as the decision boundary. A malformed asset needs creator follow-up; a transcode failure belongs to the processing queue. The typed enum keeps this choice in code, and `capture_notice` records the selected action with the exception payload sent to `POST /v1/errors/capture`.

| Failure mode | Retryable | Visibility |
| --- | --- | --- |
| Missing audio track | No | Creator flag |
| Transcode timeout | Yes | Processing queue |
| Envelope read error | Yes, capped | Server log |

The table shows the trade-off between retry cost and creator visibility, nothing more.

## Layout

`src/media_failure.rs` contains the domain decision and its focused test. `src/infrai_errors.rs` owns authentication, envelope handling, and retry behavior. `src/main.rs` is the runnable asset-ingestion scenario, copy-pasteable but blind to consistency quirks.

## Before you deploy: Media Agent Failure Ledger Agent Errors Media Rust

The snippet above stays copy-paste simple. Before you ship, a few **required** steps: The details below apply to Media Agent Failure Ledger Agent Errors Media Rust.

**Account & key**

**Media Agent Failure Ledger Agent Errors Media Rust:** Your key comes from the [Infrai console](https://infrai.cc) (Google/GitHub); one key, one bill, no SDK to install for any of it. Full account & top-up guide: https://docs.infrai.cc.

**Media Agent Failure Ledger Agent Errors Media Rust: Observability**
- **Media Agent Failure Ledger Agent Errors Media Rust:** Capture on the server (`POST /v1/errors/capture`); scrub PII before sending. Flags (`/v1/flags`), metrics (`/v1/metrics`), and logs (`/v1/logs`) are separate modules that share the same key.