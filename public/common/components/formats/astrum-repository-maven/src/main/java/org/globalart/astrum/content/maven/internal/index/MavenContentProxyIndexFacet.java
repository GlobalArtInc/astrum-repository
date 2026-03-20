
package org.globalart.astrum.content.maven.internal.index;

import java.io.IOException;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.groups.Default;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationFacet;
import org.globalart.astrum.repository.maven.MavenIndexFacet;
import org.globalart.astrum.repository.maven.internal.MavenIndexPublisher;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategy;
import org.globalart.astrum.repository.maven.internal.filter.DuplicateDetectionStrategyProvider;
import org.globalart.astrum.repository.types.ProxyType;

import com.google.common.annotations.VisibleForTesting;
import org.apache.maven.index.reader.Record;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Proxy implementation of {@link MavenIndexFacet}.
 *
 * @since 3.26
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenContentProxyIndexFacet
    extends MavenContentIndexFacetSupport
{
  static final String CONFIG_KEY = "maven-indexer";

  @VisibleForTesting
  static class Config
  {
    @NotNull(groups = ProxyType.ValidationGroup.class)
    public Boolean cacheFallback = Boolean.FALSE;

    @Override
    public String toString() {
      return getClass().getSimpleName() + "{" +
          "cacheFallback=" + cacheFallback +
          '}';
    }
  }

  private Config config;

  private final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider;

  @Autowired
  public MavenContentProxyIndexFacet(
      final MavenIndexPublisher mavenIndexPublisher,
      final DuplicateDetectionStrategyProvider duplicateDetectionStrategyProvider)
  {
    super(mavenIndexPublisher);
    this.duplicateDetectionStrategyProvider = checkNotNull(duplicateDetectionStrategyProvider);
  }

  @Override
  protected void doValidate(final Configuration configuration) {
    facet(ConfigurationFacet.class).validateSection(configuration, CONFIG_KEY, MavenContentProxyIndexFacet.Config.class,
        Default.class, getRepository().getType().getValidationGroup());
  }

  @Override
  protected void doConfigure(final Configuration configuration) {
    config = facet(ConfigurationFacet.class)
        .readSection(configuration, CONFIG_KEY, MavenContentProxyIndexFacet.Config.class);
    log.debug("Config: {}", config);
  }

  @Override
  public void publishIndex() throws IOException {
    log.debug("Fetching maven index properties from remote");
    try (DuplicateDetectionStrategy<Record> strategy = duplicateDetectionStrategyProvider.getObject()) {
      mavenIndexPublisher.publishProxyIndex(getRepository(), config.cacheFallback, strategy);
    }
  }
}
