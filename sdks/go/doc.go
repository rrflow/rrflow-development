// Package rrd is the Go client for RRFlow's public RRD protocol.
//
// The package is an outward client of the single RrdEngine authority. It does
// not implement storage, graph, index, reasoning, context, rrflowQL, or
// DataFusion behavior. Public calls carry semantic intent to the engine and
// return the engine's public protocol representation.
package rrd
