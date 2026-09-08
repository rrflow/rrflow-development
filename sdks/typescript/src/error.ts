/** Current public client error vocabulary. */

interface ApiErrorBody {
  code: string;
  message: string;
  retryable: boolean;
  details?: object;
}

export class RrdClientError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "RrdClientError";
  }
}

export class RrdApiError extends RrdClientError {
  readonly status: number;
  readonly code: string;
  readonly retryable: boolean;
  readonly details: object | undefined;

  constructor(status: number, error: ApiErrorBody) {
    super(`RRD API ${status}: ${error.code}: ${error.message}`);
    this.name = "RrdApiError";
    this.status = status;
    this.code = error.code;
    this.retryable = error.retryable;
    this.details = error.details;
  }
}
