
package org.globalart.astrum.repository.composer.internal.group;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.composer.internal.ComposerJsonProcessor;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

import java.io.IOException;
import java.time.OffsetDateTime;
import java.util.List;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Handler for merging package JSON files together.
 */
@Component
public class ComposerGroupPackageJsonHandler
    extends ComposerGroupMergingHandler
{
  private final ComposerJsonProcessor composerJsonProcessor;

  @Autowired
  public ComposerGroupPackageJsonHandler(final ComposerJsonProcessor composerJsonProcessor) {
    this.composerJsonProcessor = checkNotNull(composerJsonProcessor);
  }

  @Override
  protected Content merge(final Repository repository, final List<Payload> payloads) throws IOException {
    return composerJsonProcessor.mergePackageJson(repository, payloads, OffsetDateTime.now());
  }
}
