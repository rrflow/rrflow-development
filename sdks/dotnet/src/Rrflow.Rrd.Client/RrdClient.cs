using System;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Rrflow.Rrd;

public sealed class RrdClient : IDisposable
{
    private readonly string _instance;
    private readonly HttpTransport _transport;
    private readonly OperationExecutor _operations;

    public RrdClient(RrdClientOptions options, HttpMessageHandler? handler = null)
    {
        ClientOptions validated = ClientOptions.Validate(options);
        EndpointResolver endpoints = new(validated);
        _instance = validated.Instance;
        _transport = new HttpTransport(validated.MaxResponseBytes, handler);
        _operations = new OperationExecutor(
            endpoints,
            _transport,
            new RetryPolicy(validated));
    }

    public async Task<JsonElement> CapabilitiesAsync(
        CancellationToken cancellationToken = default)
    {
        JsonElement result = await CallAsync(
            OperationId.CapabilitiesRead,
            null,
            new RequestOptions(),
            cancellationToken).ConfigureAwait(false);
        if (result.GetProperty("protocol").GetString() != "rrd"
            || result.GetProperty("protocol_version").GetInt32() != 1
            || result.GetProperty("instance").GetProperty("id").GetString() != _instance)
        {
            throw new RrdClientException("RRD capability protocol or instance identity differs");
        }
        return result;
    }

    public Task<JsonElement> EndpointCatalogueAsync(
        CancellationToken cancellationToken = default) =>
        CallAsync(OperationId.EndpointCatalogue, null, new RequestOptions(), cancellationToken);

    public Task<JsonElement> OpenApiAsync(CancellationToken cancellationToken = default) =>
        CallAsync(OperationId.OpenapiRead, null, new RequestOptions(), cancellationToken);

    public async Task<Session> CreateSessionAsync(
        string principalId,
        string credential,
        object payload,
        RequestOptions options,
        CancellationToken cancellationToken = default)
    {
        string principal = EndpointResolver.CanonicalIdentifier(principalId, "principal");
        if (string.IsNullOrEmpty(credential))
        {
            throw new RrdClientException("API-key credential must not be empty");
        }
        JsonElement result = await CallAsync(
            OperationId.SessionCreate,
            payload,
            options with
            {
                Session = null,
                ApiKey = new ApiKeyCredentials(principal, credential),
            },
            cancellationToken).ConfigureAwait(false);
        string sessionId = result.GetProperty("session_id").GetString() ?? string.Empty;
        string token = result.GetProperty("token").GetString() ?? string.Empty;
        if (sessionId.Length == 0 || token.Length == 0)
        {
            throw new RrdClientException("invalid RRD session lease identity");
        }
        return new Session(principal, new SessionLease(sessionId, token));
    }

    public Task<JsonElement> CallAsync(
        OperationId operation,
        object? payload,
        RequestOptions options,
        CancellationToken cancellationToken = default) =>
        _operations.ExecuteAsync(operation, payload, options, cancellationToken);

    public void Dispose() => _transport.Dispose();
}
