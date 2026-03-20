
package org.globalart.astrum.content.maven.internal.index;

import java.io.IOException;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.maven.MavenIndexFacet;
import org.globalart.astrum.repository.maven.internal.MavenIndexPublisher;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategy;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategyProvider;

import org.apache.maven.index.reader.Record;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Group implementation of {@link MavenIndexFacet}.
 *
 * @since 3.26
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenContentGroupIndexFacet
    extends MavenContentIndexFacetSupport
{
  private final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider;

  @Autowired
  public MavenContentGroupIndexFacet(
      final MavenIndexPublisher mavenIndexPublisher,
      final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider)
  {
    super(mavenIndexPublisher);
    this.duplicateDetectionStrategyProvider = duplicateDetectionStrategyProvider;
  }

  @Override
  public void publishIndex() throws IOException {
    try (DuplicateDetectionStrategy<Record> strategy = duplicateDetectionStrategyProvider.getObject()) {
      mavenIndexPublisher.publishGroupIndex(getRepository(), facet(GroupFacet.class).leafMembers(), strategy);
    }
  }
}
