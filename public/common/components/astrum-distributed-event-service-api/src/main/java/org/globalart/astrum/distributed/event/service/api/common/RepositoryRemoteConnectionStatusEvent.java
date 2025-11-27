
package org.globalart.astrum.distributed.event.service.api.common;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.distributed.event.service.api.EventType.UPDATED;

/**
 * Indicates repository is auto-blocked or unblocked.
 *
 * @since 3.41
 */
public class RepositoryRemoteConnectionStatusEvent
    extends DistributedEventSupport
{
  public static final String NAME = "RepositoryRemoteConnectionStatusEvent";

  private final String repositoryName;

  private final int remoteConnectionStatusTypeOrdinal;

  private final String reason;

  private final long blockedUntilMillis;

  private final String requestUrl;

  @JsonCreator
  public RepositoryRemoteConnectionStatusEvent(
      @JsonProperty("repositoryName") final String repositoryName,
      @JsonProperty("remoteConnectionStatusTypeOrdinal") final int remoteConnectionStatusTypeOrdinal,
      @JsonProperty("reason") final String reason,
      @JsonProperty("blockedUntilMillis") final long blockedUntilMillis,
      @JsonProperty("requestUrl") final String requestUrl)
  {
    super(UPDATED);
    this.repositoryName = checkNotNull(repositoryName);
    this.remoteConnectionStatusTypeOrdinal = remoteConnectionStatusTypeOrdinal;
    this.reason = reason;
    this.blockedUntilMillis = blockedUntilMillis;
    this.requestUrl = requestUrl;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public String getReason() {
    return reason;
  }

  public int getRemoteConnectionStatusTypeOrdinal() {
    return remoteConnectionStatusTypeOrdinal;
  }

  public long getBlockedUntilMillis() {
    return blockedUntilMillis;
  }

  public String getRequestUrl() {
    return requestUrl;
  }

  @Override
  public String toString() {
    return "RepositoryRemoteConnectionStatusEvent{" +
        "repositoryName='" + repositoryName + '\'' + "," +
        "reason='" + reason + '\'' +
        '}';
  }
}
