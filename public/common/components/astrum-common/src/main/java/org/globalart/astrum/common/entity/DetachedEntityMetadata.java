
package org.globalart.astrum.common.entity;

import java.io.Serializable;
import java.util.Optional;

import javax.annotation.Nonnull;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Detached {@link EntityMetadata}.
 *
 * @since 3.0
 */
public class DetachedEntityMetadata
    implements EntityMetadata, Serializable
{
  private static final long serialVersionUID = 1L;

  private final DetachedEntityId id;

  private final DetachedEntityVersion version;

  public DetachedEntityMetadata(final DetachedEntityId id, final DetachedEntityVersion version) {
    this.id = checkNotNull(id);
    this.version = checkNotNull(version);
  }

  @Override
  @Nonnull
  public EntityId getId() {
    return id;
  }

  @Override
  @Nonnull
  public EntityVersion getVersion() {
    return version;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id=" + id +
        ", version=" + version +
        '}';
  }

  @Override
  public <T> Optional<Class<T>> getEntityType() {
    return Optional.empty();
  }

  @Override
  public <T extends Entity> Optional<T> getEntity() {
    return Optional.empty();
  }
}
