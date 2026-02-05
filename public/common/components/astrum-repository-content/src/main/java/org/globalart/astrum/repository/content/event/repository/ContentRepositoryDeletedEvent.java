
package org.globalart.astrum.repository.content.event.repository;

import org.globalart.astrum.repository.content.ContentRepository;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event sent whenever a {@link ContentRepository} is deleted.
 *
 * @since 3.26
 */
public class ContentRepositoryDeletedEvent
    extends ContentRepositoryEvent
{
  private final String format;

  public ContentRepositoryDeletedEvent(final ContentRepository contentRepository, final String format) {
    super(contentRepository);
    this.format = checkNotNull(format);
  }

  @Override
  public String getFormat() {
    return format;
  }
}
