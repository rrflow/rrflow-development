using System;
using System.Collections.Generic;
using System.Net;
using System.Text.RegularExpressions;

namespace Rrflow.Rrd;

internal sealed partial class EndpointResolver
{
    private readonly Uri _baseUri;

    internal EndpointResolver(ClientOptions options)
    {
        _baseUri = options.BaseUri;
        Instance = options.Instance;
    }

    internal string Instance { get; }

    internal Uri Resolve(Endpoint endpoint, IReadOnlyDictionary<string, string> parameters)
    {
        string path = PathParameter().Replace(endpoint.Path, match =>
        {
            string name = match.Groups[1].Value;
            if (!parameters.TryGetValue(name, out string? value) || value.Length == 0)
            {
                throw new RrdClientException($"missing path parameter {name}");
            }
            return Uri.EscapeDataString(CorrelationIdentifier(value, name + " path parameter"));
        });
        return new Uri(_baseUri, path.TrimStart('/'));
    }

    internal static string CorrelationIdentifier(string? value, string label)
    {
        if (value is null
            || value.Length is < 1 or > 128
            || !CorrelationPattern().IsMatch(value))
        {
            throw new RrdClientException($"{label} is not a canonical RRD correlation ID");
        }
        return value;
    }

    internal static string CanonicalIdentifier(string? value, string label)
    {
        if (value is null
            || value.Length is < 1 or > 128
            || !CanonicalPattern().IsMatch(value))
        {
            throw new RrdClientException($"{label} is not a canonical RRD identifier");
        }
        return value;
    }

    internal static Uri RequireLoopbackBaseUri(Uri uri)
    {
        bool loopback = uri.Host.Equals("localhost", StringComparison.OrdinalIgnoreCase)
            || (IPAddress.TryParse(uri.Host, out IPAddress? address) && IPAddress.IsLoopback(address));
        if (uri.Scheme != Uri.UriSchemeHttp
            || !loopback
            || !string.IsNullOrEmpty(uri.UserInfo)
            || !string.IsNullOrEmpty(uri.Query)
            || !string.IsNullOrEmpty(uri.Fragment))
        {
            throw new RrdClientException(
                "RRD .NET client permits only credential-free loopback HTTP before TLS qualification");
        }
        return new Uri(uri.AbsoluteUri.TrimEnd('/') + "/", UriKind.Absolute);
    }

    [GeneratedRegex("^[A-Za-z0-9._:-]+$", RegexOptions.CultureInvariant)]
    private static partial Regex CorrelationPattern();

    [GeneratedRegex("^[a-z0-9][a-z0-9._-]*$", RegexOptions.CultureInvariant)]
    private static partial Regex CanonicalPattern();

    [GeneratedRegex("\\{([a-z]+)\\}", RegexOptions.CultureInvariant)]
    private static partial Regex PathParameter();
}
