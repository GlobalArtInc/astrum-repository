
package org.globalart.astrum.common.entity;

import javax.annotation.Nonnull;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * {@link Entity} helpers.
 *
 * @since 3.0
 */
public class EntityHelper
{
  private EntityHelper() {
    // empty
  }

  /**
   * Check if given entity has metadata.
   */
  public static boolean hasMetadata(final Entity entity) {
    checkNotNull(entity);
    return entity.getEntityMetadata() != null;
  }

  /**
   * Returns metadata for entity.
   */
  @Nonnull
  public static EntityMetadata metadata(final Entity entity) {
    checkNotNull(entity);
    EntityMetadata metadata = entity.getEntityMetadata();
    checkState(metadata != null, "Missing entity-metadata");
    return metadata;
  }

  /**
   * Check if given entity is detached.
   */
  public static boolean isDetached(final Entity entity) {
    return metadata(entity) instanceof DetachedEntityMetadata;
  }

  /**
   * Returns id of entity.
   */
  @Nonnull
  public static EntityId id(final Entity entity) {
    EntityId id = metadata(entity).getId();
    // sanity id should never be null
    checkState(id != null, "Missing entity-id");
    return id;
  }

  /**
   * @param id
   * @return a DetachedEntityId
   */
  @Nonnull
  public static EntityId id(final String id) {
    return new DetachedEntityId(id);
  }

  /**
   * Returns version of entity.
   */
  @Nonnull
  public static EntityVersion version(final Entity entity) {
    EntityVersion version = metadata(entity).getVersion();
    // sanity version should never be null
    checkState(version != null, "Missing entity-version");
    return version;
  }

  /**
   * @since 3.20
   */
  public static void clearMetadata(final Object entity) {
    if (entity instanceof Entity) {
      ((Entity) entity).setEntityMetadata(null);
    }
  }
}
