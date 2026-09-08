package rrd

import "strconv"

// APIError is the current server-denial representation. H-04 owns its
// convergence into the closed, bounded, phase-aware public error hierarchy.
type APIError struct {
	Status    int
	Code      string            `json:"code"`
	Message   string            `json:"message"`
	Retryable bool              `json:"retryable"`
	Details   map[string]string `json:"details"`
}

func (e *APIError) Error() string {
	return "RRD API " + statusText(e.Status) + ": " + e.Code + ": " + e.Message
}

func statusText(status int) string {
	return strconv.Itoa(status)
}
