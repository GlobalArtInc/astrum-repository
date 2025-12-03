
package org.globalart.astrum.httpclient.config;

import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.crypto.secrets.Secret;

import com.fasterxml.jackson.annotation.JsonIgnore;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

/**
 * NTLM authentication configuration.
 *
 * @since 3.0
 */
public class NtlmAuthenticationConfiguration
    extends AuthenticationConfiguration
{
  public static final String TYPE = "ntlm";

  @NotBlank
  private String username;

  @NotNull
  private Secret password;

  @Nullable
  private String host;

  @Nullable
  private String domain;

  public NtlmAuthenticationConfiguration() {
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

  @Nullable
  public String getHost() {
    return host;
  }

  public void setHost(@Nullable final String host) {
    this.host = host;
  }

  @Nullable
  public String getDomain() {
    return domain;
  }

  public void setDomain(@Nullable final String domain) {
    this.domain = domain;
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
        ", host='" + host + '\'' +
        ", domain='" + domain + '\'' +
        '}';
  }
}
