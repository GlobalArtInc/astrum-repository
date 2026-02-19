
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model specifying the HTTP connection used by a proxy repository.
 *
 * @since 3.20
 */
public class HttpClientConnectionAttributes
{
  @Schema(description = "Total retries if the initial connection attempt suffers a timeout", example = "0")
  @Min(0L)
  @Max(10L)
  protected final Integer retries;

  @Schema(description = "Custom fragment to append to User-Agent header in HTTP requests", example = "")
  protected final String userAgentSuffix;

  @Schema(description = "Seconds to wait for activity before stopping and retrying the connection", example = "60")
  @Min(1L)
  @Max(3600L)
  protected final Integer timeout;

  @Schema(description = "Whether to enable redirects to the same location (may be required by some servers)",
      example = "false")
  protected final Boolean enableCircularRedirects;

  @Schema(description = "Whether to allow cookies to be stored and used", example = "false")
  protected final Boolean enableCookies;

  @Schema(
      description = "Use certificates stored in the Astrum Repository Manager truststore to connect to external systems",
      example = "false")
  protected final Boolean useTrustStore;

  @JsonCreator
  public HttpClientConnectionAttributes(
      @JsonProperty("retries") final Integer retries,
      @JsonProperty("userAgentSuffix") final String userAgentSuffix,
      @JsonProperty("timeout") final Integer timeout,
      @JsonProperty("enableCircularRedirects") final Boolean enableCircularRedirects,
      @JsonProperty("enableCookies") final Boolean enableCookies,
      @JsonProperty("useTrustStore") final Boolean useTrustStore)
  {
    this.retries = retries;
    this.userAgentSuffix = userAgentSuffix;
    this.timeout = timeout;
    this.enableCircularRedirects = enableCircularRedirects;
    this.enableCookies = enableCookies;
    this.useTrustStore = useTrustStore;
  }

  public Integer getRetries() {
    return retries;
  }

  public String getUserAgentSuffix() {
    return userAgentSuffix;
  }

  public Integer getTimeout() {
    return timeout;
  }

  public Boolean getEnableCircularRedirects() {
    return enableCircularRedirects;
  }

  public Boolean getEnableCookies() {
    return enableCookies;
  }

  public Boolean getUseTrustStore() {
    return useTrustStore;
  }
}
