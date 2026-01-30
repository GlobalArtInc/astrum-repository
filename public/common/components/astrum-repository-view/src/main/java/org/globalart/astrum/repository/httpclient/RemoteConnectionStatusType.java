
package org.globalart.astrum.repository.httpclient;

/**
 * Connection status of a remote repository
 *
 * @since 3.3
 */
public enum RemoteConnectionStatusType
{
  UNINITIALISED("Uninitialised"),
  READY("Ready to Connect"),
  AVAILABLE("Remote Available"),
  BLOCKED("Remote Manually Blocked"),
  AUTO_BLOCKED_UNAVAILABLE("Remote Auto Blocked and Unavailable"),
  UNAVAILABLE("Remote Unavailable"),
  OFFLINE("Repository Offline"),
  FAILED("Failed to Initialize");

  private final String description;

  RemoteConnectionStatusType(final String description) {
    this.description = description;
  }

  public String getDescription() {
    return description;
  }
}
