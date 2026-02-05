
package org.globalart.astrum.repository.content.event.repository;

import org.globalart.astrum.repository.content.ContentRepository;

/**
 * Event sent just before a {@link ContentRepository} is deleted.
 *
 * @since 3.27
 */
public class ContentRepositoryPreDeleteEvent
    extends ContentRepositoryEvent
{
  public ContentRepositoryPreDeleteEvent(final ContentRepository contentRepository) {
    super(contentRepository);
  }
}
