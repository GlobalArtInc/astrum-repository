# Monitoring and logs

## Health endpoints

| Endpoint | Authentication | Returns 200 when |
|---|---|---|
| `GET /service/rest/v1/status` | none | The server is up and can serve reads |
| `GET /service/rest/v1/status/writable` | none | The server can also accept writes; use it for load balancer and container health checks |
| `GET /service/rest/v1/status/check` | `as-metrics-all` or admin | Always 200; the body is a JSON object of named health checks with `healthy` and `message` |

The same health checks are shown under **Settings → Support → Status**.

## Prometheus

`/service/metrics/prometheus` exposes JVM, Jetty, database pool and repository metrics in the Prometheus
text format. It requires the privilege `as-metrics-all`; create a local user with only that privilege
for the scraper:

```yaml
scrape_configs:
  - job_name: astrum
    metrics_path: /service/metrics/prometheus
    scheme: https
    basic_auth:
      username: prometheus
      password_file: /etc/prometheus/astrum-password
    static_configs:
      - targets: ["astrum.example.com"]
```

Worth alerting on:

- `/service/rest/v1/status/writable` not returning 200;
- free space on the volumes of the data directory and the blob stores;
- heap usage close to the maximum for a sustained period;
- the database pool's pending connections staying above zero;
- failed scheduled tasks (see the task log below).

## Logs

| File | Content |
|---|---|
| `<data>/log/astrum.log` | Application log |
| `<data>/log/request.log` | One line per HTTP request, in the combined log format |
| `<data>/log/tasks/*.log` | One file per task run |
| `<data>/log/audit/audit.log` | Configuration and security changes, when auditing is enabled |
| `<data>/log/outbound-request.log` | Requests to remote repositories |

The container also writes the application log to standard output, so `podman logs` and
`journalctl` show it.

Log levels can be changed at runtime under **Settings → Support → Logging**.

## Support bundle

**Settings → Support → Support ZIP** collects logs, configuration and system information into one
archive, with passwords removed, for troubleshooting.
