# pretense

Server that listens on arbitary TCP ports and logs connections.
Connections are immediately closed once opened.

It logs connection attempts to stdout.
Additionally, prometheus metrics are exposed on `/metrics`.

## Configuration

Configuration is done via environment variables:
- `PRETENSE_PORTS` (required): Comma-separated list of TCP ports (u16 integers) to listen on.
  Example: `23,1234`
- `PRETENSE_METRICS_PORT`: TCP Port (u16 integer) to serve the `/metrics` HTTP endpoint on.
  If not set, no metrics are served.
  Example: `2000`

## Note about privileged ports

When listening on a privileged port (on UNIX-like systems, 1-1023), you need permissions to bind on that port.
On Linux, this can be achieved by giving the binary the `CAP_NET_BIND_SERVICE` capability.
