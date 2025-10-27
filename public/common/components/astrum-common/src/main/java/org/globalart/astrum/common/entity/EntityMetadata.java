
package org.globalart.astrum.common.entity;

import java.util.Optional;

import javax.annotation.Nonnull;

/**
 * Entity metadata.
 *
 * @since 3.0
 */
public interface EntityMetadata
{
  @Nonnull
  EntityId getId();

  @Nonnull
  EntityVersion getVersion();

  /**
   * @since 3.1
   */
  <T> Optional<Class<T>> getEntityType();

  /**
   * @since 3.1
   */
  <T extends Entity> Optional<T> getEntity();
}
