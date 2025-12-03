
package org.globalart.astrum.httpclient.config;

import javax.annotation.Nullable;
import jakarta.validation.Valid;

import org.globalart.astrum.validation.constraint.PortNumber;

import jakarta.validation.constraints.NotBlank;

/**
 * Proxy-server configuration.
 *
 * @since 3.0
 */
public class ProxyServerConfiguration
    implements Cloneable
{
  private boolean enabled;

  @NotBlank
  private String host;

  @PortNumber
  private int port;

  /**
   * @see AuthenticationConfigurationDeserializer
   */
  @Valid
  @Nullable
  private AuthenticationConfiguration authentication;

  public boolean isEnabled() {
    return enabled;
  }

  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  public String getHost() {
    return host;
  }

  public void setHost(final String host) {
    this.host = host;
  }

  public int getPort() {
    return port;
  }

  public void setPort(final int port) {
    this.port = port;
  }

  @Nullable
  public AuthenticationConfiguration getAuthentication() {
    return authentication;
  }

  public void setAuthentication(@Nullable final AuthenticationConfiguration authentication) {
    this.authentication = authentication;
  }

  public ProxyServerConfiguration copy() {
    try {
      ProxyServerConfiguration copy = (ProxyServerConfiguration) clone();
      if (authentication != null) {
        copy.authentication = authentication.copy();
      }
      return copy;
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "enabled=" + enabled +
        ", host='" + host + '\'' +
        ", port=" + port +
        ", authentication=" + authentication +
        '}';
  }
}
