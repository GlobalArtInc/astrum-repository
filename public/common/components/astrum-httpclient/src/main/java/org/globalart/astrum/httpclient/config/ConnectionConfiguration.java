
package org.globalart.astrum.httpclient.config;

import javax.annotation.Nullable;

import org.globalart.astrum.common.time.Time;

/**
 * Connection configuration.
 *
 * @since 3.0
 */
public class ConnectionConfiguration
    implements Cloneable
{
  @Nullable
  private Time timeout;

  @Nullable
  private Integer retries;

  @Nullable
  private String userAgentSuffix;

  @Nullable
  private Boolean useTrustStore;

  @Nullable
  private Boolean enableCircularRedirects;

  @Nullable
  private Boolean enableCookies;

  @Nullable
  public Time getTimeout() {
    return timeout;
  }

  public void setTimeout(@Nullable final Time timeout) {
    this.timeout = timeout;
  }

  @Nullable
  public Integer getRetries() {
    return retries;
  }

  public void setRetries(@Nullable final Integer retries) {
    this.retries = retries;
  }

  @Nullable
  public String getUserAgentSuffix() {
    return userAgentSuffix;
  }

  public void setUserAgentSuffix(@Nullable final String userAgentSuffix) {
    this.userAgentSuffix = userAgentSuffix;
  }

  @Nullable
  public Boolean getUseTrustStore() {
    return useTrustStore;
  }

  public void setUseTrustStore(@Nullable final Boolean useTrustStore) {
    this.useTrustStore = useTrustStore;
  }

  /**
   * @since 3.2.1
   */
  @Nullable
  public Boolean getEnableCircularRedirects() {
    return enableCircularRedirects;
  }

  /**
   * @since 3.2.1
   */
  public void setEnableCircularRedirects(@Nullable final Boolean enableCircularRedirects) {
    this.enableCircularRedirects = enableCircularRedirects;
  }

  /**
   * @since 3.2.1
   */
  @Nullable
  public Boolean getEnableCookies() {
    return enableCookies;
  }

  /**
   * @since 3.2.1
   */
  public void setEnableCookies(@Nullable final Boolean enableCookies) {
    this.enableCookies = enableCookies;
  }

  public ConnectionConfiguration copy() {
    try {
      return (ConnectionConfiguration) clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "timeout=" + timeout +
        ", retries=" + retries +
        ", userAgentSuffix=" + userAgentSuffix +
        ", useTrustStore=" + useTrustStore +
        ", enableCircularRedirects=" + enableCircularRedirects +
        ", enableCookies=" + enableCookies +
        '}';
  }
}
