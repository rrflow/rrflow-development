using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;

namespace Rrflow.Rrd;

internal static class ProtocolCodec
{
    private static readonly JsonSerializerOptions SerializerOptions = new(JsonSerializerDefaults.Web);

    internal static byte[]? EncodeRequest(OperationBinding binding, object? payload)
    {
        if (binding.Endpoint.Method == "GET")
        {
            return null;
        }
        return JsonSerializer.SerializeToUtf8Bytes(new Dictionary<string, object?>
        {
            ["protocol"] = "rrd",
            ["protocol_version"] = 1,
            ["context"] = binding.RequestContext,
            ["resource"] = new Dictionary<string, object?>
            {
                ["segments"] = binding.Resource,
            },
            ["payload"] = payload ?? new Dictionary<string, object?>(),
        }, SerializerOptions);
    }

    internal static JsonElement DecodeResponse(
        int status,
        byte[] encoded,
        IReadOnlyDictionary<string, object?>? requestContext)
    {
        try
        {
            using JsonDocument document = JsonDocument.Parse(encoded);
            JsonElement envelope = document.RootElement;
            RequireFields(
                envelope,
                "protocol",
                "protocol_version",
                "request_id",
                "operation_id",
                "outcome");
            if (envelope.GetProperty("protocol").GetString() != "rrd"
                || envelope.GetProperty("protocol_version").GetInt32() != 1)
            {
                throw new RrdClientException("RRD response protocol differs");
            }
            if (requestContext is not null
                && (envelope.GetProperty("request_id").GetString()
                        != (string)requestContext["request_id"]!
                    || envelope.GetProperty("operation_id").GetString()
                        != (string)requestContext["operation_id"]!))
            {
                throw new RrdClientException("RRD response request/operation identity differs");
            }
            JsonElement outcome = envelope.GetProperty("outcome");
            string outcomeStatus = outcome.GetProperty("status").GetString() ?? string.Empty;
            bool success = status is >= 200 and < 300;
            if (success != (outcomeStatus == "ok"))
            {
                throw new RrdClientException("RRD HTTP status and typed outcome disagree");
            }
            if (outcomeStatus == "error")
            {
                RequireFields(outcome, "status", "error");
                JsonElement error = outcome.GetProperty("error");
                bool hasDetails = error.TryGetProperty("details", out JsonElement detailsElement);
                RequireFields(
                    error,
                    hasDetails
                        ? new[] { "code", "message", "retryable", "details" }
                        : new[] { "code", "message", "retryable" });
                Dictionary<string, string> details = hasDetails
                    ? detailsElement.EnumerateObject().ToDictionary(
                        property => property.Name,
                        property => property.Value.GetString() ?? string.Empty,
                        StringComparer.Ordinal)
                    : new(StringComparer.Ordinal);
                throw new RrdApiException(
                    status,
                    error.GetProperty("code").GetString() ?? string.Empty,
                    error.GetProperty("message").GetString() ?? string.Empty,
                    error.GetProperty("retryable").GetBoolean(),
                    details);
            }
            if (outcomeStatus != "ok")
            {
                throw new RrdClientException("RRD response outcome status is invalid");
            }
            RequireFields(outcome, "status", "payload");
            JsonElement result = outcome.GetProperty("payload");
            if (result.ValueKind != JsonValueKind.Object)
            {
                throw new RrdClientException("RRD success payload must be an object");
            }
            return result.Clone();
        }
        catch (RrdClientException)
        {
            throw;
        }
        catch (Exception error) when (error is JsonException or InvalidOperationException)
        {
            throw new RrdClientException("RRD response envelope is invalid", error);
        }
    }

    private static void RequireFields(JsonElement element, params string[] expected)
    {
        if (element.ValueKind != JsonValueKind.Object)
        {
            throw new RrdClientException("RRD response object is invalid");
        }
        HashSet<string> actual = element.EnumerateObject()
            .Select(property => property.Name)
            .ToHashSet(StringComparer.Ordinal);
        if (!actual.SetEquals(expected))
        {
            throw new RrdClientException("RRD response object fields differ");
        }
    }
}
