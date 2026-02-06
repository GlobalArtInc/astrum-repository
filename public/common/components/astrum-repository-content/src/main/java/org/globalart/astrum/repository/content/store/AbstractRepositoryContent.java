
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.util.HashMap;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.content.RepositoryContent;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Common {@link RepositoryContent} data backed by the content data store.
 *
 * @since 3.20
 */
public abstract class AbstractRepositoryContent
    implements RepositoryContent
{
  Integer repositoryId; // NOSONAR: internal repository id

  private NestedAttributesMap attributes = new NestedAttributesMap("attributes", new HashMap<>());

  private OffsetDateTime created;

  private OffsetDateTime lastUpdated;

  // RepositoryContent API

  @Override
  public NestedAttributesMap attributes() {
    return attributes;
  }

  @Override
  public OffsetDateTime created() {
    return created;
  }

  @Override
  public OffsetDateTime lastUpdated() {
    return lastUpdated;
  }

  // MyBatis setters + validation

  /**
   * Sets the internal repository id.
   */
  public void setRepositoryId(final int repositoryId) {
    this.repositoryId = repositoryId;
  }

  /**
   * Sets the content attributes.
   */
  public void setAttributes(final NestedAttributesMap attributes) {
    this.attributes = checkNotNull(attributes);
  }

  /**
   * Sets when this metadata was first created.
   */
  public void setCreated(final OffsetDateTime created) {
    this.created = checkNotNull(created);
  }

  /**
   * Sets when this metadata was last updated.
   */
  public void setLastUpdated(final OffsetDateTime lastUpdated) {
    this.lastUpdated = checkNotNull(lastUpdated);
  }

  @Override
  public String toString() {
    return "AbstractRepositoryContent{" +
        "repositoryId=" + repositoryId +
        ", attributes=" + attributes +
        ", created=" + created +
        ", lastUpdated=" + lastUpdated +
        '}';
  }
}
