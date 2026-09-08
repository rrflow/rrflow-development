package rrd

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
)

type responseEnvelope struct {
	Protocol        string          `json:"protocol"`
	ProtocolVersion int             `json:"protocol_version"`
	RequestID       string          `json:"request_id"`
	OperationID     string          `json:"operation_id"`
	Outcome         responseOutcome `json:"outcome"`
}

type responseOutcome struct {
	Status  string          `json:"status"`
	Payload json.RawMessage `json:"payload,omitempty"`
	Error   *APIError       `json:"error,omitempty"`
}

func decodeResponse(
	status int,
	encoded []byte,
	requestContext *requestEnvelopeContext,
) (map[string]any, error) {
	var envelope responseEnvelope
	if err := decodeStrict(encoded, &envelope); err != nil {
		return nil, fmt.Errorf("RRD response envelope is invalid: %w", err)
	}
	if envelope.Protocol != "rrd" || envelope.ProtocolVersion != 1 {
		return nil, errors.New("RRD response protocol differs")
	}
	if requestContext != nil && (envelope.RequestID != requestContext.RequestID ||
		envelope.OperationID != requestContext.OperationID) {
		return nil, errors.New("RRD response request/operation identity differs")
	}
	success := status >= 200 && status < 300
	if success != (envelope.Outcome.Status == "ok") {
		return nil, errors.New("RRD HTTP status and typed outcome disagree")
	}
	if envelope.Outcome.Status == "error" {
		if envelope.Outcome.Error == nil || envelope.Outcome.Payload != nil {
			return nil, errors.New("RRD error outcome is missing its error")
		}
		if envelope.Outcome.Error.Code == "" || envelope.Outcome.Error.Message == "" {
			return nil, errors.New("RRD error outcome is incomplete")
		}
		if envelope.Outcome.Error.Details == nil {
			envelope.Outcome.Error.Details = map[string]string{}
		}
		envelope.Outcome.Error.Status = status
		return nil, envelope.Outcome.Error
	}
	if envelope.Outcome.Status != "ok" {
		return nil, errors.New("RRD response outcome status is invalid")
	}
	if envelope.Outcome.Error != nil || envelope.Outcome.Payload == nil {
		return nil, errors.New("RRD success outcome is invalid")
	}
	var payload map[string]any
	if err := decodeStrict(envelope.Outcome.Payload, &payload); err != nil || payload == nil {
		return nil, errors.New("RRD success payload must be an object")
	}
	return payload, nil
}

func decodeStrict(encoded []byte, output any) error {
	decoder := json.NewDecoder(bytes.NewReader(encoded))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(output); err != nil {
		return err
	}
	if err := decoder.Decode(&struct{}{}); err != io.EOF {
		return errors.New("JSON contains trailing data")
	}
	return nil
}

func readBounded(response *http.Response, maximum int64) ([]byte, error) {
	defer response.Body.Close()
	if response.ContentLength > maximum {
		return nil, errors.New("RRD response exceeded the configured byte limit")
	}
	encoded, err := io.ReadAll(io.LimitReader(response.Body, maximum+1))
	if err != nil {
		return nil, fmt.Errorf("read RRD response: %w", err)
	}
	if int64(len(encoded)) > maximum {
		return nil, errors.New("RRD response exceeded the configured byte limit")
	}
	return encoded, nil
}
