
package org.globalart.astrum.repository.maven.internal.filter;

import java.util.HashSet;
import java.util.Set;

import org.apache.maven.index.reader.Record;

import static com.google.common.hash.Hashing.md5;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.maven.internal.utils.RecordUtils.gavceForRecord;

/**
 * Detects duplicates using an in memory map of hashes calculated from GAV-CE. This has an upper limit and will struggle
 * on most machines to check for duplicates on central. It is, however, guaranteed to be correct and is quick.
 *
 * @since 3.11
 */
public class HashBasedDuplicateDetectionStrategy
    implements DuplicateDetectionStrategy<Record>
{
  private Set<String> gavces = new HashSet<>();

  @Override
  public boolean apply(final Record record) {
    return gavces.add(md5().hashString(gavceForRecord(record), UTF_8).toString());
  }
}
