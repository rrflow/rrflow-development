package io.rrflow.rrd;

import java.io.IOException;
import java.net.URI;
import java.net.http.HttpRequest;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import tools.jackson.databind.JsonNode;

/** Current synchronous encode/send/decode coordinator; it owns no engine state. */
final class OperationExecutor {
    private final EndpointResolver endpoints;
    private final OperationBinding binding;
    private final ProtocolCodec codec;
    private final RetryPolicy retry;
    private final HttpTransport transport;

    OperationExecutor(ClientConfig config) {
        this.endpoints = new EndpointResolver(config.baseUri());
        this.binding = new OperationBinding(config.instance());
        this.codec = new ProtocolCodec();
        this.retry = new RetryPolicy(config.requestTimeout(), config.maxAttempts());
        this.transport = new HttpTransport(config.http(), config.maxResponseBytes());
    }

    JsonNode call(OperationId operation, Object payload, RequestOptions options) {
        if (operation == null || options == null) {
            throw new RrdClientException("operation and request options are required");
        }
        Map<String, Object> context = binding.requestContext(operation, options);
        URI target = endpoints.resolve(operation, options.pathParameters());
        byte[] body = null;
        if (!operation.method().equals("GET")) {
            List<ResourceSegment> resource = binding.resource(options);
            body = codec.encodeRequest(context, resource, payload);
        }
        IOException lastError = null;
        for (int attempt = 0; attempt < retry.attempts(operation, options); attempt++) {
            Duration timeout = retry.remainingTimeout(options.deadline());
            HttpRequest.Builder builder = HttpRequest.newBuilder(target)
                    .timeout(timeout)
                    .header("Accept", "application/json");
            binding.authenticate(builder, operation, options);
            if (body == null) {
                builder.method(operation.method(), HttpRequest.BodyPublishers.noBody());
            } else {
                builder.header("Content-Type", "application/json")
                        .method(operation.method(), HttpRequest.BodyPublishers.ofByteArray(body));
            }
            try {
                HttpTransport.Response response = transport.send(builder.build());
                return codec.decodeResponse(response.status(), response.encoded(), context);
            } catch (IOException error) {
                lastError = error;
            } catch (InterruptedException error) {
                Thread.currentThread().interrupt();
                throw new RrdClientException("RRD request interrupted", error);
            }
        }
        throw new RrdClientException("RRD transport failed", lastError);
    }
}
