
package org.globalart.astrum.repository.maven.internal.utils;

import org.apache.maven.index.reader.Record;

import static org.apache.commons.lang3.StringUtils.defaultIfBlank;
import static org.apache.maven.index.reader.Record.CLASSIFIER;
import static org.apache.maven.index.reader.Record.FILE_EXTENSION;

/**
 * Provides utility methods for Maven records.
 *
 * @since 3.11
 */
public final class RecordUtils
{
  private RecordUtils() {
    // no-op
  }

  public static String gavceForRecord(final Record record) {
    String g = record.get(Record.GROUP_ID);
    String a = record.get(Record.ARTIFACT_ID);
    String v = record.get(Record.VERSION);
    String ce = defaultIfBlank(record.get(CLASSIFIER), "n/a") + ":" + record.get(FILE_EXTENSION);

    return g + a + v + ce;
  }
}
