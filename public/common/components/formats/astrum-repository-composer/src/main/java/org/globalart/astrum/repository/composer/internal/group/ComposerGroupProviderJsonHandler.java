
package org.globalart.astrum.repository.composer.internal.group;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.io.IOException;
import java.time.OffsetDateTime;
import java.util.List;


import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Handler for merging provider JSON files together.
 */
@Component
public class ComposerGroupProviderJsonHandler
    extends ComposerGroupMergingHandler
{
  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerGroupProviderJsonHandler(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Override
  protected Content merge(final Repository repository, final List<Payload> payloads) throws IOException {
    return composerJsonProcessor.mergeProviderJson(repository, payloads, OffsetDateTime.now());
  }
}
