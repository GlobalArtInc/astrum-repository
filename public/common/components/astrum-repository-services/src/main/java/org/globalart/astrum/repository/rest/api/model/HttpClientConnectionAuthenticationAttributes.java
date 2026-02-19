
package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

/**
 * REST API model for describing authentication for HTTP connections used by a proxy repository.
 *
 * @since 3.20
 */
public class HttpClientConnectionAuthenticationAttributes
{
  @Schema(description = "Authentication type")
  @NotEmpty
  protected final String type;

  @Schema
  protected final String username;

  @Schema(accessMode = Schema.AccessMode.WRITE_ONLY)
  protected final String password;

  @Schema
  protected final String ntlmHost;

  @Schema
  protected final String ntlmDomain;

  @Schema
  protected final String bearerToken;

  @JsonCreator
  public HttpClientConnectionAuthenticationAttributes(
      @JsonProperty("type") final String type,
      @JsonProperty("username") final String username,
      // ASTRUM-46395: Access.WRITE_ONLY keeps Jackson from serializing this field on outbound
      // GET responses (e.g. repository configuration listings). The @Schema(accessMode=WRITE_ONLY)
      // annotation on the field is OpenAPI documentation only and does not influence Jackson;
      // the JsonProperty access mode is the actual gate.
      @JsonProperty(value = "password", access = JsonProperty.Access.WRITE_ONLY) final String password,
      @JsonProperty("ntlmHost") final String ntlmHost,
      @JsonProperty("ntlmDomain") final String ntlmDomain,
      @JsonProperty(value = "bearerToken", access = JsonProperty.Access.WRITE_ONLY) final String bearerToken)
  {
    this.type = type;
    this.username = username;
    this.password = password;
    this.ntlmHost = ntlmHost;
    this.ntlmDomain = ntlmDomain;
    this.bearerToken = bearerToken;
  }

  public String getType() {
    return type;
  }

  public String getUsername() {
    return username;
  }

  public String getPassword() {
    return password;
  }

  public String getNtlmHost() {
    return ntlmHost;
  }

  public String getNtlmDomain() {
    return ntlmDomain;
  }

  public String getBearerToken() {
    return bearerToken;
  }

}
