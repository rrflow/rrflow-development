package io.rrflow.rrd;

import java.net.URI;
import java.net.http.HttpClient;
import java.time.Duration;

/** Current validated construction state; H-07 owns remote endpoint profiles. */
final class ClientConfig {
    static final int DEFAULT_RESPONSE_LIMIT = 4 * 1024 * 1024;
    private static final int MAXIMUM_RESPONSE_LIMIT = 16 * 1024 * 1024;

    private final URI baseUri;
    private final String instance;
    private final Duration requestTimeout;
    private final int maxAttempts;
    private final int maxResponseBytes;
    private final HttpClient http;

    ClientConfig(
            String baseUrl,
            String instance,
            Duration requestTimeout,
            int maxAttempts,
            int maxResponseBytes,
            HttpClient http) {
        this.baseUri = EndpointResolver.loopbackUri(baseUrl);
        this.instance = OperationBinding.canonical(instance, "instance");
        if (requestTimeout.isZero()
                || requestTimeout.isNegative()
                || requestTimeout.compareTo(Duration.ofMinutes(5)) > 0) {
            throw new RrdClientException("request timeout must be in (0, 5m]");
        }
        if (maxAttempts < 1 || maxAttempts > 8) {
            throw new RrdClientException("max attempts must be in 1..=8");
        }
        if (maxResponseBytes < 1 || maxResponseBytes > MAXIMUM_RESPONSE_LIMIT) {
            throw new RrdClientException("response limit must be in 1..=16777216 bytes");
        }
        this.requestTimeout = requestTimeout;
        this.maxAttempts = maxAttempts;
        this.maxResponseBytes = maxResponseBytes;
        this.http = http == null
                ? HttpClient.newBuilder().followRedirects(HttpClient.Redirect.NEVER).build()
                : http;
    }

    URI baseUri() {
        return baseUri;
    }

    String instance() {
        return instance;
    }

    Duration requestTimeout() {
        return requestTimeout;
    }

    int maxAttempts() {
        return maxAttempts;
    }

    int maxResponseBytes() {
        return maxResponseBytes;
    }

    HttpClient http() {
        return http;
    }
}
