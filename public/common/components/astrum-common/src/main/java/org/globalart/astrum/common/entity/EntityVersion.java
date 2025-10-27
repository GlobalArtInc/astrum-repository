
package org.globalart.astrum.common.entity;

import javax.annotation.Nonnull;

/**
 * Entity version for external usage.
 *
 * @since 3.0
 */
public interface EntityVersion
{
  /**
   * External version for entity.
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
