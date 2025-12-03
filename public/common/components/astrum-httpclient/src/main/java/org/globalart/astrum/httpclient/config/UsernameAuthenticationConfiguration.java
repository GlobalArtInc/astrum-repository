
package org.globalart.astrum.httpclient.config;

import java.util.List;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.crypto.secrets.Secret;

import com.fasterxml.jackson.annotation.JsonIgnore;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

/**
 * Username(+password) authentication configuration.
 *
 * @since 3.0
 */
public class UsernameAuthenticationConfiguration
    extends AuthenticationConfiguration
{
  public static final String TYPE = "username";

  @NotBlank
  private String username;

  @NotNull
  private Secret password;

  public UsernameAuthenticationConfiguration() {
    super(TYPE);
  }

  public String getUsername() {
    return username;
  }

  public void setUsername(final String username) {
    this.username = username;
  }

  public Secret getPassword() {
    return password;
  }

  public void setPassword(final Secret password) {
    this.password = password;
  }

  @Override
  public Secret getSecret() {
    return getPassword();
  }

  @Override
  @JsonIgnore
  public List<String> getSecretFieldNames() {
    return List.of("password");
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "username='" + username + '\'' +
        ", password='" + Strings2.MASK + '\'' +
        '}';
  }
}
