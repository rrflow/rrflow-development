package io.rrflow.rrd;

import java.net.http.HttpClient;
import java.time.Duration;
import tools.jackson.databind.JsonNode;

/** Synchronous Java facade for RRFlow's public RRD protocol. */
public final class RrdClient {
    private final String instance;
    private final OperationExecutor operations;

    public RrdClient(String baseUrl, String instance) {
        this(
                baseUrl,
                instance,
                Duration.ofSeconds(5),
                2,
                ClientConfig.DEFAULT_RESPONSE_LIMIT,
                null);
    }

    public RrdClient(
            String baseUrl,
            String instance,
            Duration requestTimeout,
            int maxAttempts,
            int maxResponseBytes,
            HttpClient http) {
        ClientConfig config = new ClientConfig(
                baseUrl, instance, requestTimeout, maxAttempts, maxResponseBytes, http);
        this.instance = config.instance();
        this.operations = new OperationExecutor(config);
    }

    public JsonNode capabilities() {
        JsonNode result = call(OperationId.CAPABILITIES_READ, null, RequestOptions.empty());
        if (!result.path("protocol").asString().equals("rrd")
                || result.path("protocol_version").asInt() != 1
                || !result.path("instance").path("id").asString().equals(instance)) {
            throw new RrdClientException("RRD capability protocol or instance identity differs");
        }
        return result;
    }

    public JsonNode endpointCatalogue() {
        return call(OperationId.ENDPOINT_CATALOGUE, null, RequestOptions.empty());
    }

    public JsonNode openApi() {
        return call(OperationId.OPENAPI_READ, null, RequestOptions.empty());
    }

    public Session createSession(
            String principalId, String credential, Object payload, RequestOptions options) {
        String principal = OperationBinding.canonical(principalId, "principal");
        if (credential == null || credential.isEmpty()) {
            throw new RrdClientException("API-key credential must not be empty");
        }
        JsonNode result = call(
                OperationId.SESSION_CREATE, payload, options.withApiKey(principal, credential));
        String sessionId = result.path("session_id").asString();
        String token = result.path("token").asString();
        if (sessionId.isEmpty() || token.isEmpty()) {
            throw new RrdClientException("invalid RRD session lease identity");
        }
        return new Session(principal, new Session.SessionLease(sessionId, token));
    }

    public JsonNode call(OperationId operation, Object payload, RequestOptions options) {
        return operations.call(operation, payload, options);
    }
}
