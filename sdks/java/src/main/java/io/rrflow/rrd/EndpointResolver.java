package io.rrflow.rrd;

import java.net.InetAddress;
import java.net.URI;
import java.net.URLEncoder;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/** Current loopback endpoint and route resolution; H-07 owns installed identities. */
final class EndpointResolver {
    private static final Pattern PATH_PARAMETER = Pattern.compile("\\{([a-z]+)}");

    private final URI baseUri;

    EndpointResolver(URI baseUri) {
        this.baseUri = baseUri;
    }

    URI resolve(OperationId operation, Map<String, String> parameters) {
        String path = resolvePath(operation.path(), parameters);
        return baseUri.resolve(path.startsWith("/") ? path.substring(1) : path);
    }

    static URI loopbackUri(String value) {
        final URI uri;
        try {
            uri = URI.create(value);
            String host = uri.getHost();
            boolean loopback = "localhost".equals(host);
            if (host != null && !loopback && host.matches("^[0-9a-fA-F:.]+$")) {
                InetAddress address = InetAddress.getByName(host);
                loopback = address.isLoopbackAddress();
            }
            if (!"http".equals(uri.getScheme())
                    || !loopback
                    || uri.getUserInfo() != null
                    || uri.getQuery() != null
                    || uri.getFragment() != null) {
                throw new IllegalArgumentException();
            }
            return URI.create(value.endsWith("/") ? value : value + "/");
        } catch (Exception error) {
            throw new RrdClientException(
                    "RRD Java client permits only credential-free loopback HTTP before TLS qualification",
                    error);
        }
    }

    private static String resolvePath(String template, Map<String, String> parameters) {
        Matcher matcher = PATH_PARAMETER.matcher(template);
        StringBuilder result = new StringBuilder();
        while (matcher.find()) {
            String value = parameters.get(matcher.group(1));
            if (value == null || value.isEmpty()) {
                throw new RrdClientException("missing path parameter " + matcher.group(1));
            }
            String encoded = URLEncoder.encode(
                            OperationBinding.correlation(
                                    value, matcher.group(1) + " path parameter"),
                            StandardCharsets.UTF_8)
                    .replace("+", "%20");
            matcher.appendReplacement(result, Matcher.quoteReplacement(encoded));
        }
        matcher.appendTail(result);
        return result.toString();
    }
}
