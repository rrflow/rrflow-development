package rrd

// APIKeyCredentials is the current caller-provided API-key representation.
// H-04 owns replacement with an opaque, redacted credential provider.
type APIKeyCredentials struct {
	PrincipalID string
	Credential  string
}

// SessionLease is the current wire representation of an authenticated lease.
type SessionLease struct {
	SessionID               string         `json:"session_id"`
	Token                   string         `json:"token"`
	IssuedAtUnixMS          uint64         `json:"issued_at_unix_ms"`
	IdleExpiresAtUnixMS     uint64         `json:"idle_expires_at_unix_ms"`
	AbsoluteExpiresAtUnixMS uint64         `json:"absolute_expires_at_unix_ms"`
	Limits                  map[string]any `json:"limits"`
}

// Session is the current public session representation. Its exported bearer
// remains a documented H-04 defect; this split does not conceal or bless it.
type Session struct {
	PrincipalID string
	Lease       SessionLease
}
