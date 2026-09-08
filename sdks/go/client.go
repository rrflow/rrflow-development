package rrd

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"time"
)

// Client is the synchronous Go facade for the public RRD protocol.
type Client struct {
	baseURL          *url.URL
	instance         string
	requestTimeout   time.Duration
	maxAttempts      int
	maxResponseBytes int64
	http             *http.Client
}

// NewClient validates the current loopback profile and constructs a reusable
// concurrent client. Authenticated remote construction remains gated to H-07.
func NewClient(config Config) (*Client, error) {
	validated, err := validateConfig(config)
	if err != nil {
		return nil, err
	}
	return &Client{
		baseURL:          validated.baseURL,
		instance:         validated.instance,
		requestTimeout:   validated.requestTimeout,
		maxAttempts:      validated.maxAttempts,
		maxResponseBytes: validated.maxResponseBytes,
		http: &http.Client{
			Transport: validated.transport,
			CheckRedirect: func(_ *http.Request, _ []*http.Request) error {
				return errors.New("RRD redirects are disabled")
			},
		},
	}, nil
}

// Capabilities reads and validates the current engine protocol and instance
// identity advertised by the public discovery operation.
func (c *Client) Capabilities(ctx context.Context) (map[string]any, error) {
	result, err := c.Call(ctx, OperationCapabilitiesRead, nil, RequestOptions{})
	if err != nil {
		return nil, err
	}
	instance, ok := result["instance"].(map[string]any)
	if result["protocol"] != "rrd" || result["protocol_version"] != float64(1) ||
		!ok || instance["id"] != c.instance {
		return nil, errors.New("RRD capability protocol or instance identity differs")
	}
	return result, nil
}

// EndpointCatalogue reads the engine's public operation catalogue.
func (c *Client) EndpointCatalogue(ctx context.Context) (map[string]any, error) {
	return c.Call(ctx, OperationEndpointCatalogue, nil, RequestOptions{})
}

// OpenAPI reads the engine's public OpenAPI representation.
func (c *Client) OpenAPI(ctx context.Context) (map[string]any, error) {
	return c.Call(ctx, OperationOpenapiRead, nil, RequestOptions{})
}

// CreateSession creates the current public bearer-bearing session value.
// H-04 owns its replacement with an opaque credential-safe handle.
func (c *Client) CreateSession(
	ctx context.Context,
	principalID string,
	credential string,
	payload any,
	options RequestOptions,
) (*Session, error) {
	principal, err := canonical(principalID, "principal")
	if err != nil {
		return nil, err
	}
	if credential == "" {
		return nil, errors.New("API-key credential must not be empty")
	}
	options.APIKey = &APIKeyCredentials{PrincipalID: principal, Credential: credential}
	result, err := c.Call(ctx, OperationSessionCreate, payload, options)
	if err != nil {
		return nil, err
	}
	raw, err := json.Marshal(result)
	if err != nil {
		return nil, fmt.Errorf("encode session lease: %w", err)
	}
	var lease SessionLease
	if err := decodeStrict(raw, &lease); err != nil {
		return nil, fmt.Errorf("invalid RRD session lease: %w", err)
	}
	if lease.SessionID == "" || lease.Token == "" {
		return nil, errors.New("invalid RRD session lease identity")
	}
	return &Session{PrincipalID: principal, Lease: lease}, nil
}

// Call invokes one generated public operation using the current generic
// payload/result surface. H-04 owns concrete operation bindings.
func (c *Client) Call(
	ctx context.Context,
	operation OperationID,
	payload any,
	options RequestOptions,
) (map[string]any, error) {
	return c.executeOperation(ctx, operation, payload, options)
}
