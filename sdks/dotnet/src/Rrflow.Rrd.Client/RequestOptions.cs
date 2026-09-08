using System;
using System.Collections.Generic;

namespace Rrflow.Rrd;

public sealed record RequestOptions
{
    public string? RequestId { get; init; }
    public string? OperationId { get; init; }
    public string? IdempotencyKey { get; init; }
    public DateTimeOffset? Deadline { get; init; }
    public IReadOnlyDictionary<string, string> PathParameters { get; init; }
        = new Dictionary<string, string>();
    public IReadOnlyList<ResourceSegment>? Resource { get; init; }
    public Session? Session { get; init; }
    public ApiKeyCredentials? ApiKey { get; init; }
}
