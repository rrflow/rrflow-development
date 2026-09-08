package io.rrflow.rrd;

import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.json.JsonMapper;

/** Current partial RRD envelope codec; H-04 owns complete generated validation. */
final class ProtocolCodec {
    private static final JsonMapper JSON = JsonMapper.builder().build();

    byte[] encodeRequest(
            Map<String, Object> context, List<ResourceSegment> resource, Object payload) {
        Map<String, Object> envelope = new HashMap<>();
        envelope.put("protocol", "rrd");
        envelope.put("protocol_version", 1);
        envelope.put("context", context);
        envelope.put("resource", Map.of("segments", resource));
        envelope.put("payload", payload == null ? Map.of() : payload);
        try {
            return JSON.writeValueAsBytes(envelope);
        } catch (Exception error) {
            throw new RrdClientException("encode RRD request", error);
        }
    }

    JsonNode decodeResponse(int status, byte[] encoded, Map<String, Object> context) {
        final JsonNode envelope;
        try {
            envelope = JSON.readTree(encoded);
        } catch (Exception error) {
            throw new RrdClientException("RRD response envelope is invalid", error);
        }
        requireExactFields(
                envelope,
                Set.of("protocol", "protocol_version", "request_id", "operation_id", "outcome"));
        if (!envelope.path("protocol").asString().equals("rrd")
                || envelope.path("protocol_version").asInt() != 1) {
            throw new RrdClientException("RRD response protocol differs");
        }
        if (context != null
                && (!envelope.path("request_id").asString().equals(context.get("request_id"))
                        || !envelope
                                .path("operation_id")
                                .asString()
                                .equals(context.get("operation_id")))) {
            throw new RrdClientException("RRD response request/operation identity differs");
        }
        JsonNode outcome = envelope.path("outcome");
        String outcomeStatus = outcome.path("status").asString();
        boolean success = status >= 200 && status < 300;
        if (success != outcomeStatus.equals("ok")) {
            throw new RrdClientException("RRD HTTP status and typed outcome disagree");
        }
        if (outcomeStatus.equals("error")) {
            requireExactFields(outcome, Set.of("status", "error"));
            JsonNode error = outcome.path("error");
            boolean hasDetails = error.has("details");
            requireExactFields(
                    error,
                    hasDetails
                            ? Set.of("code", "message", "retryable", "details")
                            : Set.of("code", "message", "retryable"));
            if (error.path("code").asString().isEmpty()
                    || error.path("message").asString().isEmpty()
                    || !error.path("retryable").isBoolean()
                    || (hasDetails && !error.path("details").isObject())) {
                throw new RrdClientException("RRD error outcome is incomplete");
            }
            Map<String, String> details = new HashMap<>();
            if (hasDetails) {
                error.path("details")
                        .properties()
                        .forEach(entry ->
                                details.put(entry.getKey(), entry.getValue().asString()));
            }
            throw new RrdApiException(
                    status,
                    error.path("code").asString(),
                    error.path("message").asString(),
                    error.path("retryable").asBoolean(),
                    details);
        }
        if (!outcomeStatus.equals("ok")) {
            throw new RrdClientException("RRD response outcome status is invalid");
        }
        requireExactFields(outcome, Set.of("status", "payload"));
        JsonNode result = outcome.path("payload");
        if (!result.isObject()) {
            throw new RrdClientException("RRD success payload must be an object");
        }
        return result;
    }

    private static void requireExactFields(JsonNode node, Set<String> expected) {
        if (node == null || !node.isObject()) {
            throw new RrdClientException("RRD response object is invalid");
        }
        Set<String> actual = new HashSet<>();
        actual.addAll(node.propertyNames());
        if (!actual.equals(expected)) {
            throw new RrdClientException("RRD response object fields differ");
        }
    }
}
