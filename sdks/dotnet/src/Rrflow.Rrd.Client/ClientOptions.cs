using System;

namespace Rrflow.Rrd;

public sealed record RrdClientOptions
{
    public required Uri BaseUri { get; init; }
    public required string Instance { get; init; }
    public TimeSpan RequestTimeout { get; init; } = TimeSpan.FromSeconds(5);
    public int MaxAttempts { get; init; } = 2;
    public int MaxResponseBytes { get; init; } = 4 * 1024 * 1024;
}

internal sealed record ClientOptions(
    Uri BaseUri,
    string Instance,
    TimeSpan RequestTimeout,
    int MaxAttempts,
    int MaxResponseBytes)
{
    private const int MaximumResponseBytes = 16 * 1024 * 1024;

    internal static ClientOptions Validate(RrdClientOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        Uri baseUri = EndpointResolver.RequireLoopbackBaseUri(options.BaseUri);
        string instance = EndpointResolver.CanonicalIdentifier(options.Instance, "instance");
        if (options.RequestTimeout <= TimeSpan.Zero
            || options.RequestTimeout > TimeSpan.FromMinutes(5))
        {
            throw new RrdClientException("request timeout must be in (0, 5m]");
        }
        if (options.MaxAttempts is < 1 or > 8)
        {
            throw new RrdClientException("max attempts must be in 1..=8");
        }
        if (options.MaxResponseBytes is < 1 or > MaximumResponseBytes)
        {
            throw new RrdClientException("response limit must be in 1..=16777216 bytes");
        }
        return new ClientOptions(
            baseUri,
            instance,
            options.RequestTimeout,
            options.MaxAttempts,
            options.MaxResponseBytes);
    }
}
