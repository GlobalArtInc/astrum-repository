
package org.globalart.astrum.repository.maven.internal.filter;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.maven.index.reader.Record;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Provides the configured record duplicate detection strategy. Defaults to
 * {@link BloomFilterDuplicateDetectionStrategy}
 */
@Component
public class DuplicateDetectionStrategyProvider
    implements FactoryBean<DuplicateDetectionStrategy<Record>>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final String strategy;

  private final ApplicationDirectories applicationDirectories;

  private final int maxHeapGb;

  private final int maxDiskSizeGb;

  @Autowired
  public DuplicateDetectionStrategyProvider(
      final ApplicationDirectories applicationDirectories,
      @Value("${astrum.maven.duplicate.detection.strategy:BLOOM}") final String strategy,
      @Value("${astrum.maven.duplicate.detection.heap.max.gb:1}") final int maxHeapGb,
      @Value("${astrum.maven.duplicate.detection.disk.max.gb:10}") final int maxDiskSizeGb)
  {
    this.applicationDirectories = checkNotNull(applicationDirectories);
    this.strategy = checkNotNull(strategy);
    this.maxHeapGb = maxHeapGb;
    this.maxDiskSizeGb = maxDiskSizeGb;
  }

  @Override
  public DuplicateDetectionStrategy<Record> getObject() {
    return switch (getStrategy()) {
      case HASH -> new HashBasedDuplicateDetectionStrategy();
      case DISK -> new DiskBackedDuplicateDetectionStrategy(applicationDirectories, maxHeapGb, maxDiskSizeGb);
      default -> new BloomFilterDuplicateDetectionStrategy();
    };
  }

  private Strategy getStrategy() {
    Strategy duplicateStrategy = null;
    try {
      duplicateStrategy = Strategy.valueOf(this.strategy.toUpperCase());
    }
    catch (IllegalArgumentException e) { // NOSONAR
      log.warn("Unsupported record duplicate detection strategy {}. Falling back to bloom. ", duplicateStrategy);
      duplicateStrategy = Strategy.BLOOM;
    }
    return duplicateStrategy;
  }

  @Override
  public Class<?> getObjectType() {
    return DuplicateDetectionStrategy.class;
  }

  private enum Strategy
  {
    HASH,
    BLOOM,
    DISK
  }
}
