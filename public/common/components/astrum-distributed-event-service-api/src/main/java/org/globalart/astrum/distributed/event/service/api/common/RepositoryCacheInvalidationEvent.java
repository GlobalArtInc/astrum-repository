
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.distributed.event.service.api.EventType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Indicates that the cache should be invalidated for provided repository
 *
 * @since 3.41
 */
public class RepositoryCacheInvalidationEvent
    extends DistributedEventSupport
{
  public static final String NAME = "RepositoryCacheInvalidationEvent";

  private final String repositoryName;

  @JsonCreator
  public RepositoryCacheInvalidationEvent(@JsonProperty("repositoryName") final String repositoryName) {
    super(EventType.UPDATED);
    this.repositoryName = repositoryName;
  }

  public String getRepositoryName() {
    return repositoryName;
  }
}
