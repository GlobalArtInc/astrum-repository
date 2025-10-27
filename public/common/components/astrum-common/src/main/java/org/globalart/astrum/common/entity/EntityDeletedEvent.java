
package org.globalart.astrum.common.entity;

/**
 * Entity deleted event.
 *
 * @since 3.1
 */
public class EntityDeletedEvent
    extends EntityEvent
{
  public EntityDeletedEvent(final EntityMetadata metadata) {
    super(metadata);
  }
}
