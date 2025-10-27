
package org.globalart.astrum.common.entity;

/**
 * Entity created event.
 *
 * @since 3.1
 */
public class EntityCreatedEvent
    extends EntityEvent
{
  public EntityCreatedEvent(final EntityMetadata metadata) {
    super(metadata);
  }
}
