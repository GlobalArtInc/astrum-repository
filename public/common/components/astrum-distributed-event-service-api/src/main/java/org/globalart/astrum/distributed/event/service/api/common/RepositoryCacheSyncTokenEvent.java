
package org.globalart.astrum.distributed.event.service.api.common;

import org.globalart.astrum.common.event.EventWithSource;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * Used to notify other nodes that cached data for the repository may be stale and needs refreshing.
 *
 * @since 3.84
 */
public class RepositoryCacheSyncTokenEvent
    extends EventWithSource
{
  public static final String NAME = "RepositoryCacheSyncTokenEvent";

  private final String repositoryName;

  private final String cacheToken;

  @JsonCreator
  public RepositoryCacheSyncTokenEvent(
      @JsonProperty("repositoryName") final String repositoryName,
      @JsonProperty("cacheToken") final String cacheToken)
  {
    this.repositoryName = repositoryName;
    this.cacheToken = cacheToken;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public String getToken() {
    return cacheToken;
  }
}
