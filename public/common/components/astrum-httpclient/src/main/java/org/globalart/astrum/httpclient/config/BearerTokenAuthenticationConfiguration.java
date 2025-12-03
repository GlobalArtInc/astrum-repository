
package org.globalart.astrum.httpclient.config;

import java.util.List;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.crypto.secrets.Secret;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Bearer Token authentication configuration
 *
 * @since 3.20
 */
public class BearerTokenAuthenticationConfiguration
    extends AuthenticationConfiguration
{
  public static final String TYPE = "bearerToken";

  @NotNull
  @JsonProperty("bearerTokenId")
  private Secret bearerToken;

  public BearerTokenAuthenticationConfiguration() {
    super(TYPE);
  }

  public Secret getBearerToken() {
    return bearerToken;
  }

  public void setBearerToken(final Secret bearerToken) {
    this.bearerToken = bearerToken;
  }

  @Override
  public Secret getSecret() {
    return getBearerToken();
  }

  @Override
  @JsonIgnore
  public List<String> getSecretFieldNames() {
    return List.of("bearerTokenId");
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "bearerToken='" + Strings2.MASK + '\'' +
        '}';
  }
}
