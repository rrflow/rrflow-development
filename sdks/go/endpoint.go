package rrd

import (
	"errors"
	"fmt"
	"net"
	"net/url"
	"strings"
)

func loopbackURL(value string) (*url.URL, error) {
	parsed, err := url.Parse(value)
	if err != nil {
		return nil, fmt.Errorf("parse RRD URL: %w", err)
	}
	host := parsed.Hostname()
	loopback := host == "localhost"
	if address := net.ParseIP(host); address != nil {
		loopback = address.IsLoopback()
	}
	if parsed.Scheme != "http" || !loopback || parsed.User != nil || parsed.RawQuery != "" ||
		parsed.Fragment != "" {
		return nil, errors.New(
			"RRD Go client permits only credential-free loopback HTTP before TLS qualification",
		)
	}
	if !strings.HasSuffix(parsed.Path, "/") {
		parsed.Path += "/"
	}
	return parsed, nil
}
