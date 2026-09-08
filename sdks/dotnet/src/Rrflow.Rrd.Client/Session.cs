namespace Rrflow.Rrd;

public sealed record SessionLease(string SessionId, string Token);

public sealed record Session(string PrincipalId, SessionLease Lease);

public sealed record ApiKeyCredentials(string PrincipalId, string Credential);
