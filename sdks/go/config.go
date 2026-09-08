package rrd

import (
	"errors"
	"net/http"
	"net/url"
	"time"
)

const (
	defaultResponseLimit = int64(4 * 1024 * 1024)
	maximumResponseLimit = int64(16 * 1024 * 1024)
)

// Config contains the current validated construction inputs for Client.
// Remote authenticated endpoint profiles remain gated to H-07.
type Config struct {
	BaseURL          string
	Instance         string
	RequestTimeout   time.Duration
	MaxAttempts      int
	MaxResponseBytes int64
	Transport        http.RoundTripper
}

type validatedConfig struct {
	baseURL          *url.URL
	instance         string
	requestTimeout   time.Duration
	maxAttempts      int
	maxResponseBytes int64
	transport        http.RoundTripper
}

func validateConfig(config Config) (*validatedConfig, error) {
	baseURL, err := loopbackURL(config.BaseURL)
	if err != nil {
		return nil, err
	}
	instance, err := canonical(config.Instance, "instance")
	if err != nil {
		return nil, err
	}
	timeout := config.RequestTimeout
	if timeout == 0 {
		timeout = 5 * time.Second
	}
	if timeout < time.Millisecond || timeout > 5*time.Minute {
		return nil, errors.New("request timeout must be in [1ms, 5m]")
	}
	attempts := config.MaxAttempts
	if attempts == 0 {
		attempts = 2
	}
	if attempts < 1 || attempts > 8 {
		return nil, errors.New("max attempts must be in 1..=8")
	}
	limit := config.MaxResponseBytes
	if limit == 0 {
		limit = defaultResponseLimit
	}
	if limit < 1 || limit > maximumResponseLimit {
		return nil, errors.New("response limit must be in 1..=16777216 bytes")
	}
	transport := config.Transport
	if transport == nil {
		transport = http.DefaultTransport
	}
	return &validatedConfig{
		baseURL:          baseURL,
		instance:         instance,
		requestTimeout:   timeout,
		maxAttempts:      attempts,
		maxResponseBytes: limit,
		transport:        transport,
	}, nil
}
