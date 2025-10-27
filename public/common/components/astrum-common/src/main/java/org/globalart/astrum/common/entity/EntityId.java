
package org.globalart.astrum.common.entity;

import javax.annotation.Nonnull;

import com.fasterxml.jackson.databind.annotation.JsonDeserialize;

/**
 * Entity identifier.
 *
 * @since 3.0
 */
@JsonDeserialize(as = EntityUUID.class)
public interface EntityId
{
  /**
   * External identifier for entity.
   */
  @Nonnull
  String getValue();

  /**
   * Returns human-readable representation for logging.
   *
   * @see #getValue() for the external representation.
   */
  @Override
  String toString();
}
