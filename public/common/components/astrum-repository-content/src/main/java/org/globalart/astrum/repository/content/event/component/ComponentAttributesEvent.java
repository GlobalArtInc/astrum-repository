
package org.globalart.astrum.repository.content.event.component;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;

/**
 * Event sent whenever a {@link Component}'s attributes change.
 *
 * @since 3.26
 */
public class ComponentAttributesEvent
    extends ComponentUpdatedEvent
{
  private final AttributeOperation change;

  private final String key;

  @Nullable
  private final Object value;

  public ComponentAttributesEvent(
      final Component component,
      final AttributeOperation change,
      final String key,
      @Nullable final Object value)
  {
    super(component);
    this.change = checkNotNull(change);
    this.key = checkNotNull(key);
    this.value = value;
  }

  public AttributeOperation getChange() {
    return change;
  }

  public String getKey() {
    return key;
  }

  @SuppressWarnings("unchecked")
  public <T> Optional<T> getValue() {
    return ofNullable((T) value);
  }

  @Override
  public String toString() {
    return "ComponentAttributesEvent{" +
        "change=" + change +
        ", key='" + key + '\'' +
        ", value=" + value +
        "} " + super.toString();
  }
}
