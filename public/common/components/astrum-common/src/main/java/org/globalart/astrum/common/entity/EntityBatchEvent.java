
package org.globalart.astrum.common.entity;

import java.util.Collections;
import java.util.List;

import org.globalart.astrum.common.event.HasAffinity;

/**
 * Batched sequence of {@link EntityEvent}s.
 *
 * @since 3.1
 */
public class EntityBatchEvent
    implements HasAffinity
{
  /**
   * Marker for {@link EntityEvent}s which can be batched together in a {@link EntityBatchEvent} as well
   * as being sent individually. Useful if subscribers want to receive a group of events at the same time.
   */
  public interface Batchable
  {
    // empty
  }

  private final List<EntityEvent> events;

  public EntityBatchEvent(final List<EntityEvent> events) {
    this.events = Collections.unmodifiableList(events);
  }

  public List<EntityEvent> getEvents() {
    return events;
  }

  @Override
  public String getAffinity() {
    return events.get(0).getAffinity(); // first event in the batch declares the affinity for the rest
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "events=" + events +
        '}';
  }
}
