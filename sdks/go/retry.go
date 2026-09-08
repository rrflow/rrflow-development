package rrd

import (
	"context"
	"errors"
	"net/http"
	"time"
)

func (c *Client) attemptLimit(endpoint Endpoint, options RequestOptions) int {
	if endpoint.Method == http.MethodGet || !endpoint.Mutation || options.IdempotencyKey != "" {
		return c.maxAttempts
	}
	return 1
}

func (c *Client) attemptContext(
	ctx context.Context,
	deadline time.Time,
) (context.Context, context.CancelFunc, error) {
	remaining := c.requestTimeout
	if !deadline.IsZero() {
		until := time.Until(deadline)
		if until <= 0 {
			return nil, nil, errors.New("RRD request deadline has expired")
		}
		if until < remaining {
			remaining = until
		}
	}
	attempt, cancel := context.WithTimeout(ctx, remaining)
	return attempt, cancel, nil
}
