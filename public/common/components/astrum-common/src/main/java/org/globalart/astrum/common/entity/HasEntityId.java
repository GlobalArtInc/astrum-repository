
package org.globalart.astrum.common.entity;

import com.fasterxml.jackson.annotation.JsonIgnore;

/**
 * Mix-in for entities that don't have a natural key and need a generated id.
 *
 * @since 3.19
 */
public interface HasEntityId
{
  EntityId getId();

  void setId(EntityId id);

  /**
   * Entity which should be serialized/deserialized with {@link EntityId} should implement this method.
   *
   * @return {@code true} to allow {@link EntityId} be generated or {@code false} otherwise.
   */
  @JsonIgnore
  default boolean allowGenerate() {
    return true;
  }

  default void clearId() {
    setId(null);
  }
}
