
package org.globalart.astrum.repository.httpclient;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Represents a change in repository status.
 *
 * @since 3.3
 */
public class RemoteConnectionStatusEvent
    extends RepositoryEvent
{
  private final RemoteConnectionStatus status;

  public RemoteConnectionStatusEvent(final RemoteConnectionStatus status, final Repository repository) {
    super(repository);
    this.status = checkNotNull(status);
  }

  public RemoteConnectionStatus getStatus() {
    return status;
  }

  @Override
  public String toString() {
    return "RemoteConnectionStatusEvent{" +
        "status=" + getStatus() +
        ", repository=" + getRepository() +
        '}';
  }
}
