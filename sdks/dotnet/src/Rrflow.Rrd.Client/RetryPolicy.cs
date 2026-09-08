using System;

namespace Rrflow.Rrd;

internal sealed class RetryPolicy
{
    private readonly TimeSpan _requestTimeout;
    private readonly int _maxAttempts;

    internal RetryPolicy(ClientOptions options)
    {
        _requestTimeout = options.RequestTimeout;
        _maxAttempts = options.MaxAttempts;
    }

    internal int AttemptsFor(Endpoint endpoint, RequestOptions options)
    {
        bool retrySafe = endpoint.Method == "GET"
            || !endpoint.Mutation
            || options.IdempotencyKey is not null;
        return retrySafe ? _maxAttempts : 1;
    }

    internal TimeSpan RemainingTimeout(DateTimeOffset? deadline)
    {
        if (deadline is null)
        {
            return _requestTimeout;
        }
        TimeSpan remaining = deadline.Value - DateTimeOffset.UtcNow;
        if (remaining <= TimeSpan.Zero)
        {
            throw new RrdClientException("RRD request deadline has expired");
        }
        return remaining < _requestTimeout ? remaining : _requestTimeout;
    }
}
