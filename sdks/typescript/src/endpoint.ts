/** Current credential-free loopback endpoint profile. */

import { RrdClientError } from "./error.js";

export interface ClientConfig {
  baseUrl: string;
  instance: string;
  requestTimeoutMs?: number;
  maxAttempts?: number;
  maxResponseBytes?: number;
  fetch?: typeof globalThis.fetch;
}

export function loopbackUrl(value: string): URL {
  const url = new URL(value);
  if (
    url.protocol !== "http:" ||
    !["localhost", "127.0.0.1", "[::1]"].includes(url.hostname) ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    throw new RrdClientError(
      "RRD TypeScript client permits only credential-free loopback HTTP before TLS qualification",
    );
  }
  if (!url.pathname.endsWith("/")) url.pathname += "/";
  return url;
}
