
package org.globalart.astrum.repository.content.event.repository;

import org.globalart.astrum.common.event.EventWithSource;
import org.globalart.astrum.repository.content.ContentRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent whenever a {@link ContentRepository}'s attributes change between all nodes.
 */
public class ContentRepositoryAttributesDesEvent
    extends EventWithSource
{
  private final String repoName;

  @JsonCreator
  public ContentRepositoryAttributesDesEvent(@JsonProperty("repoName") final String repoName) {
    this.repoName = checkNotNull(repoName);
  }

  public String getRepoName() {
    return repoName;
  }
}
