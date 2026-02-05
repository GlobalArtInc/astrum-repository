
package org.globalart.astrum.repository.content.event.repository;

import org.globalart.astrum.repository.content.ContentRepository;

/**
 * Event sent whenever a {@link ContentRepository} is created.
 *
 * @since 3.26
 */
public class ContentRepositoryCreatedEvent
    extends ContentRepositoryEvent
{
  public ContentRepositoryCreatedEvent(final ContentRepository contentRepository) {
    super(contentRepository);
  }
}
