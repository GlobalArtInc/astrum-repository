
package org.globalart.astrum.common.entity;

import javax.annotation.Nullable;

import org.globalart.astrum.common.event.HasAffinity;
import org.globalart.astrum.common.event.HasLocality;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Entity event.
 *
 * @since 3.1
 */
@SuppressWarnings("unchecked")
public abstract class EntityEvent
    implements HasAffinity, HasLocality
{
  private final EntityMetadata metadata;

  private String remoteNodeId;

  private String affinity;

  private volatile Entity entity;

  public EntityEvent(final EntityMetadata metadata) {
    this.metadata = checkNotNull(metadata);
  }

  public EntityId getId() {
    return metadata.getId();
  }

  public EntityVersion getVersion() {
    return metadata.getVersion();
  }

  /**
   * @return {@code true} if this is a local event
   *
   * @since 3.1
   */
  @Override
  public boolean isLocal() {
    return remoteNodeId == null;
  }

  /**
   * @param remoteNodeId the remote node that sent this event; {@code null} if event is local
   *
   * @since 3.1
   */
  public void setRemoteNodeId(@Nullable final String remoteNodeId) {
    this.remoteNodeId = remoteNodeId;
  }

  /**
   * @return the remote node that sent this event; {@code null} if event is local
   *
   * @since 3.1
   */
  @Nullable
  public String getRemoteNodeId() {
    return remoteNodeId;
  }

  /**
   * @return attached entity type, if it exists
   */
  @Nullable
  public <T extends Entity> Class<T> getEntityType() {
    return metadata.<T>getEntityType().orElse(null);
  }

  /**
   * @return attached entity, if it exists
   */
  @Nullable
  public <T extends Entity> T getEntity() {
    // can be expensive depending on the entity, so use lazy evaluation
    if (entity == null) {
      synchronized (this) {
        if (entity == null) {
          entity = metadata.getEntity().orElse(null);
        }
      }
    }
    return (T) entity;
  }

  @Override
  public String getAffinity() {
    return affinity;
  }

  /**
   * Declares the affinity for this event.
   *
   * @since 3.11
   */
  public void setAffinity(final String affinity) {
    this.affinity = affinity;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "metadata=" + metadata +
        ", remoteNodeId=" + remoteNodeId +
        '}';
  }
}
