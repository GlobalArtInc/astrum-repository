
package org.globalart.astrum.repository.maven.rest;

import org.globalart.astrum.repository.rest.api.model.HttpClientConnectionAuthenticationAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * REST API model for describing authentication for HTTP connections used by a proxy repository supporting preemptive
 * authentication.
 *
 * @since 3.30
 */
public class HttpClientConnectionAuthenticationAttributesWithPreemptive
    extends HttpClientConnectionAuthenticationAttributes
{
  @Schema(description = "Whether to use pre-emptive authentication. Use with caution. Defaults to false.",
      example = "false")
  protected final Boolean preemptive;

  @JsonCreator
  public HttpClientConnectionAuthenticationAttributesWithPreemptive(
      @JsonProperty("type") final String type,
      @JsonProperty("preemptive") final Boolean preemptive,
      @JsonProperty("username") final String username,
      // ASTRUM-46395: see HttpClientConnectionAuthenticationAttributes for why Access.WRITE_ONLY
      // is needed here - it's what actually keeps Jackson from leaking the credential on outbound
      // serialization. @Schema(accessMode=WRITE_ONLY) is OpenAPI-doc only.
      @JsonProperty(value = "password", access = JsonProperty.Access.WRITE_ONLY) final String password,
      @JsonProperty("ntlmHost") final String ntlmHost,
      @JsonProperty("ntlmDomain") final String ntlmDomain,
      @JsonProperty(value = "bearerToken", access = JsonProperty.Access.WRITE_ONLY) final String bearerToken)
  {
    super(type, username, password, ntlmHost, ntlmDomain, bearerToken);
    this.preemptive = preemptive;
  }

  public HttpClientConnectionAuthenticationAttributesWithPreemptive(
      final HttpClientConnectionAuthenticationAttributes auth,
      final Boolean preemptive)
  {
    super(auth.getType(), auth.getUsername(), auth.getPassword(), auth.getNtlmHost(), auth.getNtlmDomain(),
        auth.getBearerToken());
    this.preemptive = preemptive;
  }

  public Boolean isPreemptive() {
    return preemptive;
  }
}
