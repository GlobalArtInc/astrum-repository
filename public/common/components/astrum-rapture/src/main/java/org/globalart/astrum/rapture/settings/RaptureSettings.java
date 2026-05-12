
package org.globalart.astrum.rapture.settings;

import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotBlank;

/**
 * Rapture settings.
 *
 * @since 3.0
 */
public class RaptureSettings
{
  public static final String DEFAULT_TITLE = "GlobalArt Astrum Repository";

  public static final boolean DEFAULT_DEBUG_ALLOWED = true;

  public static final int DEFAULT_STATUS_INTERVAL_AUTHENTICATED = 5; // seconds

  public static final int DEFAULT_STATUS_INTERVAL_ANONYMOUS = 60; // seconds

  public static final int DEFAULT_SESSION_TIMEOUT = 30; // minutes

  public static final int MIN_REQUEST_TIMEOUT = 30; // seconds

  public static final int DEFAULT_REQUEST_TIMEOUT = 60; // seconds

  public static final int DEFAULT_LONG_REQUEST_TIMEOUT = 3 * 60; // seconds

  public static final int MIN_SEARCH_REQUEST_TIMEOUT = 0; // seconds

  private boolean debugAllowed = DEFAULT_DEBUG_ALLOWED;

  @Min(0)
  private int statusIntervalAuthenticated = DEFAULT_STATUS_INTERVAL_AUTHENTICATED;

  @Min(0)
  private int statusIntervalAnonymous = DEFAULT_STATUS_INTERVAL_ANONYMOUS;

  /**
   * Session timeout in minutes. Set to 0 for sessions that never expire.
   * Note: This UI setting can be overridden by astrum.jwt.expiry (for JWT mode)
   * or shiro.globalSessionTimeout (for session mode) properties if explicitly set.
   * Warning: Zero timeout (no expiration) may be a security risk.
   */
  @Min(0)
  private int sessionTimeout = DEFAULT_SESSION_TIMEOUT;

  @Min(MIN_REQUEST_TIMEOUT)
  private int requestTimeout = DEFAULT_REQUEST_TIMEOUT;

  @Min(DEFAULT_LONG_REQUEST_TIMEOUT)
  private int longRequestTimeout = DEFAULT_LONG_REQUEST_TIMEOUT;

  @NotBlank
  private String title;

  public boolean isDebugAllowed() {
    return debugAllowed;
  }

  public void setDebugAllowed(final boolean debugAllowed) {
    this.debugAllowed = debugAllowed;
  }

  public int getStatusIntervalAuthenticated() {
    return statusIntervalAuthenticated;
  }

  public void setStatusIntervalAuthenticated(final int statusInterval) {
    this.statusIntervalAuthenticated = statusInterval;
  }

  public int getStatusIntervalAnonymous() {
    return statusIntervalAnonymous;
  }

  public void setStatusIntervalAnonymous(final int statusInterval) {
    this.statusIntervalAnonymous = statusInterval;
  }

  public int getSessionTimeout() {
    return sessionTimeout;
  }

  public void setSessionTimeout(final int sessionTimeout) {
    this.sessionTimeout = sessionTimeout;
  }

  public int getRequestTimeout() {
    return requestTimeout;
  }

  public void setRequestTimeout(final int requestTimeout) {
    this.requestTimeout = requestTimeout;
  }

  public int getLongRequestTimeout() {
    return longRequestTimeout;
  }

  public void setLongRequestTimeout(final int longRequestTimeout) {
    this.longRequestTimeout = longRequestTimeout;
  }

  public String getTitle() {
    return title;
  }

  public void setTitle(final String title) {
    this.title = title;
  }
}
