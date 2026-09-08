using System;
using System.Buffers;
using System.IO;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;

namespace Rrflow.Rrd;

internal sealed record HttpTransportResponse(int StatusCode, byte[] EncodedBody);

internal sealed class HttpTransport : IDisposable
{
    private readonly HttpClient _http;
    private readonly int _maxResponseBytes;

    internal HttpTransport(int maxResponseBytes, HttpMessageHandler? handler)
    {
        _maxResponseBytes = maxResponseBytes;
        _http = handler is null
            ? new HttpClient(new HttpClientHandler { AllowAutoRedirect = false })
            : new HttpClient(handler, disposeHandler: false);
    }

    internal async Task<HttpTransportResponse> SendAsync(
        HttpRequestMessage request,
        CancellationToken cancellationToken)
    {
        using HttpResponseMessage response = await _http.SendAsync(
            request,
            HttpCompletionOption.ResponseHeadersRead,
            cancellationToken).ConfigureAwait(false);
        byte[] encoded = await ReadBoundedAsync(response, cancellationToken).ConfigureAwait(false);
        return new HttpTransportResponse((int)response.StatusCode, encoded);
    }

    public void Dispose() => _http.Dispose();

    private async Task<byte[]> ReadBoundedAsync(
        HttpResponseMessage response,
        CancellationToken cancellationToken)
    {
        if (response.Content.Headers.ContentLength > _maxResponseBytes)
        {
            throw new RrdClientException("RRD response exceeded the configured byte limit");
        }
        await using Stream input = await response.Content.ReadAsStreamAsync(cancellationToken)
            .ConfigureAwait(false);
        using MemoryStream output = new();
        byte[] buffer = ArrayPool<byte>.Shared.Rent(8192);
        try
        {
            int total = 0;
            int count;
            while ((count = await input.ReadAsync(buffer, cancellationToken).ConfigureAwait(false)) > 0)
            {
                total += count;
                if (total > _maxResponseBytes)
                {
                    throw new RrdClientException("RRD response exceeded the configured byte limit");
                }
                output.Write(buffer, 0, count);
            }
            return output.ToArray();
        }
        finally
        {
            ArrayPool<byte>.Shared.Return(buffer);
        }
    }
}
