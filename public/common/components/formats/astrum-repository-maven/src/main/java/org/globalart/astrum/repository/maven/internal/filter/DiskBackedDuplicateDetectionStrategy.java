
package org.globalart.astrum.repository.maven.internal.filter;

import java.util.UUID;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;

import org.apache.maven.index.reader.Record;
import org.ehcache.Cache;
import org.ehcache.CacheManager;
import org.ehcache.config.builders.CacheManagerBuilder;
import org.ehcache.config.builders.ResourcePoolsBuilder;

import static org.ehcache.config.builders.CacheConfigurationBuilder.newCacheConfigurationBuilder;
import static org.ehcache.config.units.MemoryUnit.GB;
import static org.globalart.astrum.repository.maven.internal.utils.RecordUtils.gavceForRecord;

/**
 * Detects duplicates using a disk backed mv store which will spill over to disk when the map gets too large.
 *
 * This only has an upper limit based on the disk size but does require a temporary file to be created which might
 * not be appropriate in some environments. It is also marginally slower than other strategies.
 *
 * This strategy is recommended for large datasets where the bloom filter strategy is not appropriate because records
 * have been missed.
 *
 * @since 3.11
 */
public class DiskBackedDuplicateDetectionStrategy
    implements DuplicateDetectionStrategy<Record>
{
  public static final String CACHE_NAME = "duplicate-detection-cache";

  private final Cache<String, String> map;

  private final CacheManager cacheManager;

  public DiskBackedDuplicateDetectionStrategy(
      final ApplicationDirectories directories,
      final int maxHeapGb,
      final int maxDiskGb)
  {
    String randomDirectory = UUID.randomUUID().toString();

    cacheManager = CacheManagerBuilder.newCacheManagerBuilder()
        .with(CacheManagerBuilder.persistence(directories.getTemporaryDirectory().getPath() + "/" + randomDirectory))
        .withCache(CACHE_NAME, newCacheConfigurationBuilder(String.class, String.class,
            ResourcePoolsBuilder.newResourcePoolsBuilder()
                .heap(maxHeapGb, GB)
                .disk(maxDiskGb, GB)))
        .build(true);

    map = cacheManager.getCache(CACHE_NAME, String.class, String.class);
  }

  @Override
  public boolean apply(final Record record) {
    String gavce = gavceForRecord(record);

    if (map.containsKey(gavce)) {
      return false;
    }
    else {
      map.put(gavce, "");

      return true;
    }
  }

  @Override
  public void close() {
    cacheManager.close();
  }
}
