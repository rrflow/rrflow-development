package io.rrflow.rrd;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;

/** Current HTTP carriage and response cap; H-07/J-02 own full resource policy. */
final class HttpTransport {
    record Response(int status, byte[] encoded) { }

    private final HttpClient http;
    private final int maxResponseBytes;

    HttpTransport(HttpClient http, int maxResponseBytes) {
        this.http = http;
        this.maxResponseBytes = maxResponseBytes;
    }

    Response send(HttpRequest request) throws IOException, InterruptedException {
        HttpResponse<InputStream> response =
                http.send(request, HttpResponse.BodyHandlers.ofInputStream());
        return new Response(response.statusCode(), readBounded(response));
    }

    private byte[] readBounded(HttpResponse<InputStream> response) throws IOException {
        long declared = response.headers().firstValueAsLong("Content-Length").orElse(-1);
        if (declared > maxResponseBytes) {
            throw new RrdClientException("RRD response exceeded the configured byte limit");
        }
        try (InputStream input = response.body();
                ByteArrayOutputStream output = new ByteArrayOutputStream()) {
            byte[] buffer = new byte[8192];
            int total = 0;
            int count;
            while ((count = input.read(buffer)) != -1) {
                total += count;
                if (total > maxResponseBytes) {
                    throw new RrdClientException(
                            "RRD response exceeded the configured byte limit");
                }
                output.write(buffer, 0, count);
            }
            return output.toByteArray();
        }
    }
}
