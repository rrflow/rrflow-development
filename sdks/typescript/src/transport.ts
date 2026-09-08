/** Current bounded Fetch response carriage and envelope decoding. */

import { type } from "arktype";
import { RrdApiError, RrdClientError } from "./error.js";
import type { OperationId, RequestContext, SuccessPayload } from "./operation.js";

const apiErrorBody = type({
  code: "string",
  message: "string",
  retryable: "boolean",
  "details?": "object",
  "+": "reject",
});
const responseOutcome = type.or(
  type({ status: "'ok'", payload: "unknown", "+": "reject" }),
  type({ status: "'error'", error: apiErrorBody, "+": "reject" }),
);
const responseEnvelope = type({
  protocol: "'rrd'",
  protocol_version: "1",
  request_id: "string",
  operation_id: "string",
  outcome: responseOutcome,
  "+": "reject",
});

export async function decodeResponse<K extends OperationId>(
  response: Response,
  maximumBytes: number,
  expected: RequestContext | undefined,
): Promise<SuccessPayload<K>> {
  const encoded = await readBounded(response, maximumBytes);
  let decoded: unknown;
  try {
    decoded = JSON.parse(encoded);
  } catch (error) {
    throw new RrdClientError(
      error instanceof Error
        ? `RRD response JSON is invalid: ${error.message}`
        : "RRD response JSON is invalid",
    );
  }
  const validated = responseEnvelope(decoded);
  if (validated instanceof type.errors) {
    throw new RrdClientError(`RRD response envelope is invalid: ${validated.summary}`);
  }
  if (
    expected &&
    (validated.request_id !== expected.request_id ||
      validated.operation_id !== expected.operation_id)
  ) {
    throw new RrdClientError("RRD response request/operation identity differs");
  }
  if (response.ok !== (validated.outcome.status === "ok")) {
    throw new RrdClientError("RRD HTTP status and typed outcome disagree");
  }
  if (validated.outcome.status === "error") {
    throw new RrdApiError(response.status, validated.outcome.error);
  }
  return validated.outcome.payload as SuccessPayload<K>;
}

async function readBounded(response: Response, maximum: number): Promise<string> {
  const declared = Number(response.headers.get("content-length"));
  if (Number.isFinite(declared) && declared > maximum) {
    throw new RrdClientError("RRD response exceeded the configured byte limit");
  }
  if (!response.body) return "";
  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    length += value.byteLength;
    if (length > maximum) {
      await reader.cancel();
      throw new RrdClientError("RRD response exceeded the configured byte limit");
    }
    chunks.push(value);
  }
  const combined = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    combined.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return new TextDecoder("utf-8", { fatal: true }).decode(combined);
}
