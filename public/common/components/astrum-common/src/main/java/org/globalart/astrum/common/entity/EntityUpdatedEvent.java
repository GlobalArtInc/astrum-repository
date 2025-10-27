
package org.globalart.astrum.common.entity;

/**
 * Entity updated event.
 *
 * @since 3.1
 */
public class EntityUpdatedEvent
    extends EntityEvent
{
  public EntityUpdatedEvent(final EntityMetadata metadata) {
    super(metadata);
  }
}
