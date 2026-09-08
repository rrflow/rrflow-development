using System;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace Rrflow.Rrd;

internal sealed class OperationExecutor
{
    private readonly EndpointResolver _endpointResolver;
    private readonly HttpTransport _transport;
    private readonly RetryPolicy _retryPolicy;

    internal OperationExecutor(
        EndpointResolver endpointResolver,
        HttpTransport transport,
        RetryPolicy retryPolicy)
    {
        _endpointResolver = endpointResolver;
        _transport = transport;
        _retryPolicy = retryPolicy;
    }

    internal async Task<JsonElement> ExecuteAsync(
        OperationId operation,
        object? payload,
        RequestOptions options,
        CancellationToken cancellationToken)
    {
        ArgumentNullException.ThrowIfNull(options);
        OperationBinding binding = OperationBinding.Create(operation, options, _endpointResolver);
        byte[]? body = ProtocolCodec.EncodeRequest(binding, payload);
        int attempts = _retryPolicy.AttemptsFor(binding.Endpoint, options);
        Exception? lastError = null;
        for (int attempt = 0; attempt < attempts; attempt++)
        {
            TimeSpan timeout = _retryPolicy.RemainingTimeout(options.Deadline);
            using CancellationTokenSource attemptTimeout = new(timeout);
            using CancellationTokenSource linked = CancellationTokenSource.CreateLinkedTokenSource(
                cancellationToken,
                attemptTimeout.Token);
            using HttpRequestMessage request = new(
                new HttpMethod(binding.Endpoint.Method),
                binding.Target);
            request.Headers.Accept.Add(new MediaTypeWithQualityHeaderValue("application/json"));
            binding.Authenticate(request);
            if (body is not null)
            {
                request.Content = new ByteArrayContent(body);
                request.Content.Headers.ContentType = new MediaTypeHeaderValue("application/json");
            }
            try
            {
                HttpTransportResponse response = await _transport.SendAsync(request, linked.Token)
                    .ConfigureAwait(false);
                return ProtocolCodec.DecodeResponse(
                    response.StatusCode,
                    response.EncodedBody,
                    binding.RequestContext);
            }
            catch (Exception error) when (
                error is HttpRequestException
                || (error is OperationCanceledException
                    && !cancellationToken.IsCancellationRequested))
            {
                lastError = error;
            }
        }
        cancellationToken.ThrowIfCancellationRequested();
        throw new RrdClientException("RRD transport failed", lastError!);
    }
}
