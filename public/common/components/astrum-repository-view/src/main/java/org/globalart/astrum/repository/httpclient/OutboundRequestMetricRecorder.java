
package org.globalart.astrum.repository.httpclient;

/**
 * Interface for recording outbound HTTP request metrics from proxy repositories.
 * <p>
 * Implementations collect telemetry data about requests made to remote registries
 * (e.g., npmjs.org, Maven Central) for analytics purposes.
 */
public interface OutboundRequestMetricRecorder
{
  String CONTEXT_FORMAT = "astrum.outbound.format";

  String CONTEXT_REPOSITORY_TYPE = "astrum.outbound.repositoryType";

  /**
   * Record an outbound request.
   *
   * @param format Repository format (e.g., "maven2", "npm", "docker", "nuget")
   * @param repositoryType Repository type (e.g., "proxy")
   * @param httpMethod HTTP method (e.g., "GET", "HEAD")
   */
  void record(String format, String repositoryType, String httpMethod);
}
