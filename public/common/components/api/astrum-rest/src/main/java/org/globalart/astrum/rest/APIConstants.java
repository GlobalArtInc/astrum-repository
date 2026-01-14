
package org.globalart.astrum.rest;

/**
 * Constants relating to REST APIs.
 *
 * @since 3.4
 */
public class APIConstants
{
  private APIConstants() {
  }

  public static final String V1_API_PREFIX = "/v1";

  /**
   * Used for REST endpoints which use version 2 of the API.
   */
  public static final String V2_API_PREFIX = "/v2";

  /**
   * Used for REST endpoints which are labeled as beta. These endpoints may be changed though we ought to warn users via
   * release notes when this occurs.
   */
  public static final String BETA_API_PREFIX = "/beta";

  public static final String INTERNAL_API_PREFIX = "/internal";

  /**
   * Mime type for versionless REST API URLs which indicates that the Content-Type is V1 from the endpoint.
   *
   * @see <a href="https://swagger.io/docs/specification/2-0/mime-types/">Swagger 2.0 Mime Types</a>
   */
  public static final String APPLICATION_ASTRUM_V1 = "application/vnd.globalart.astrum.v1+json";
}
