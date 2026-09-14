/** Supported TypeScript client surface for the public RRD v1 protocol. */

export { RrdClient } from "./client.js";
export type { ClientConfig } from "./endpoint.js";
export { RrdApiError, RrdClientError } from "./error.js";
export {
  OPENAPI_DOCUMENT_SHA256,
  SIGNAL_CATALOGUE,
  SIGNAL_CATALOGUE_JSON,
  SIGNAL_CATALOGUE_SHA256,
} from "./generated/signal-catalogue.js";
export type {
  OperationId,
  operations,
  paths,
  RequestOptions,
  RequestPayload,
  ResourceSegment,
  SuccessPayload,
} from "./operation.js";
export type { Session } from "./session.js";
