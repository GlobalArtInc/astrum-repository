
package org.globalart.astrum.content.maven.internal.index;

import java.io.IOException;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.maven.MavenIndexFacet;
import org.globalart.astrum.repository.maven.internal.MavenIndexPublisher;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategy;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategyProvider;
import org.globalart.astrum.repository.maven.internal.hosted.MavenHostedIndexFacet;

import org.apache.maven.index.reader.Record;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Hosted implementation of {@link MavenIndexFacet}.
 *
 * @since 3.26
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenContentHostedIndexFacet
    extends MavenContentIndexFacetSupport
    implements MavenHostedIndexFacet
{
  private final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider;

  @Autowired
  public MavenContentHostedIndexFacet(
      final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider,
      final MavenIndexPublisher mavenIndexPublisher)
  {
    super(mavenIndexPublisher);
    this.duplicateDetectionStrategyProvider = checkNotNull(duplicateDetectionStrategyProvider);
  }

  @Override
  public void publishIndex() throws IOException {
    try (DuplicateDetectionStrategy<Record> strategy = duplicateDetectionStrategyProvider.getObject()) {
      mavenIndexPublisher.publishHostedIndex(getRepository(), strategy);
    }
  }
}
