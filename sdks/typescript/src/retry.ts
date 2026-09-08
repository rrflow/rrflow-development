/** Current attempt/deadline helpers pending semantic outcome certainty. */

import { RrdClientError } from "./error.js";

export function boundedInteger(
  value: number,
  minimum: number,
  maximum: number,
  label: string,
): number {
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw new RrdClientError(`${label} must be an integer in ${minimum}..=${maximum}`);
  }
  return value;
}

export function remainingTimeout(configured: number, deadline: number | undefined): number {
  if (deadline === undefined) return configured;
  const remaining = deadline - Date.now();
  if (remaining <= 0) throw new RrdClientError("RRD request deadline has expired");
  return Math.min(configured, remaining);
}

export function attemptLimit(
  method: string,
  mutation: boolean,
  idempotencyKey: string | undefined,
  configured: number,
): number {
  return method === "GET" || !mutation || idempotencyKey ? configured : 1;
}
