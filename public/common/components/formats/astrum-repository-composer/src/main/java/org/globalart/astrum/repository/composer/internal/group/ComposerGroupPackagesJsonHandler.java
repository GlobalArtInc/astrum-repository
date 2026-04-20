
package org.globalart.astrum.repository.composer.internal.group;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.util.List;


import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Handler for merging packages.json files together.
 */
@Component
public class ComposerGroupPackagesJsonHandler
    extends ComposerGroupMergingHandler
{
  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerGroupPackagesJsonHandler(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Override
  protected Content merge(final Repository repository, final List<Payload> payloads) throws Exception {
    return composerJsonProcessor.mergePackagesJson(repository, payloads);
  }
}
