
package org.globalart.astrum.cleanup.internal;

import java.util.Objects;

import org.globalart.astrum.cleanup.content.search.CleanupBrowseServiceFactory;
import org.globalart.astrum.cleanup.content.search.CleanupComponentBrowse;
import org.globalart.astrum.cleanup.content.search.DefaultCleanupComponentBrowse;
import org.globalart.astrum.repository.Format;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

@Lazy
@Component
public class DefaultCleanupBrowseServiceFactory
    implements CleanupBrowseServiceFactory
{
  private final DefaultCleanupComponentBrowse defaultCleanupComponentBrowse;

  @Autowired
  public DefaultCleanupBrowseServiceFactory(final DefaultCleanupComponentBrowse defaultCleanupComponentBrowse) {
    this.defaultCleanupComponentBrowse = Objects.requireNonNull(defaultCleanupComponentBrowse);
  }

  @Override
  public CleanupComponentBrowse get(final Format format) {
    return defaultCleanupComponentBrowse;
  }

  @Override
  public CleanupComponentBrowse getPreviewService() {
    return defaultCleanupComponentBrowse;
  }
}
