package io.rrflow.rrd;

import java.time.Duration;
import java.time.Instant;

/** Current broad replay and deadline behavior; H-04 owns semantic certainty. */
final class RetryPolicy {
    private final Duration requestTimeout;
    private final int maxAttempts;

    RetryPolicy(Duration requestTimeout, int maxAttempts) {
        this.requestTimeout = requestTimeout;
        this.maxAttempts = maxAttempts;
    }

    int attempts(OperationId operation, RequestOptions options) {
        boolean retrySafe = operation.method().equals("GET")
                || !operation.mutation()
                || options.idempotencyKey() != null;
        return retrySafe ? maxAttempts : 1;
    }

    Duration remainingTimeout(Instant deadline) {
        if (deadline == null) {
            return requestTimeout;
        }
        Duration remaining = Duration.between(Instant.now(), deadline);
        if (remaining.isZero() || remaining.isNegative()) {
            throw new RrdClientException("RRD request deadline has expired");
        }
        return remaining.compareTo(requestTimeout) < 0 ? remaining : requestTimeout;
    }
}
