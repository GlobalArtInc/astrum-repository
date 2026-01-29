
package org.globalart.astrum.repository;

import org.globalart.astrum.common.collect.NestedAttributesMap;

/**
 * A command that modifies a {@link Repository}'s attributes.
 *
 * @since 3.0
 */
public interface AttributeChange
{
  /**
   * Operations must have no side effects as they may be retried by the transaction infrastructure.
   */
  void apply(NestedAttributesMap attributes);
}
