
package org.globalart.astrum.repository.content.event.repository;

import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.ContentRepository;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;

/**
 * Event sent whenever a {@link ContentRepository}'s attributes change.
 *
 * @since 3.26
 */
public class ContentRepositoryAttributesEvent
    extends ContentRepositoryUpdatedEvent
{
  private final AttributeOperation change;

  private final String key;

  @Nullable
  private final Object value;

  public ContentRepositoryAttributesEvent(
      final ContentRepository contentRepository,
      final AttributeOperation change,
      final String key,
      @Nullable final Object value)
  {
    super(contentRepository);
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
}
