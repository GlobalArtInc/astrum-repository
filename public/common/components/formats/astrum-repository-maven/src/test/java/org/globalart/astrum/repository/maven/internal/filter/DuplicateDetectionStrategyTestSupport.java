
package org.globalart.astrum.repository.maven.internal.filter;

import com.google.common.collect.ImmutableMap;
import org.apache.maven.index.reader.Record;
import org.junit.Rule;
import org.mockito.junit.MockitoJUnit;
import org.mockito.junit.MockitoRule;

import static org.apache.maven.index.reader.Record.ARTIFACT_ID;
import static org.apache.maven.index.reader.Record.CLASSIFIER;
import static org.apache.maven.index.reader.Record.FILE_EXTENSION;
import static org.apache.maven.index.reader.Record.GROUP_ID;
import static org.apache.maven.index.reader.Record.Type.ARTIFACT_ADD;
import static org.apache.maven.index.reader.Record.VERSION;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class DuplicateDetectionStrategyTestSupport
{
  @Rule
  public MockitoRule mockitoRule = MockitoJUnit.rule().silent();

  public void verifyDuplicateDetection(final DuplicateDetectionStrategy<Record> strategy) throws Exception {
    int expectedUnique = 3;

    for (int i = 1; i <= expectedUnique; i++) {
      Record uniqueRecord = buildRecord("group1", "artifact" + i, "1.0", "sources", "jar");
      assertTrue(strategy.apply(uniqueRecord));
    }

    for (int i = 0; i < 10; i++) {
      Record duplicateRecord = buildRecord("group1", "artifact" + expectedUnique, "1.0", "sources", "jar");
      assertFalse(strategy.apply(duplicateRecord));
    }

    strategy.close();
  }

  private Record buildRecord(
      final String g,
      final String a,
      final String v,
      final String c,
      final String e)
  {
    return new Record(ARTIFACT_ADD, ImmutableMap.of(GROUP_ID, g,
        ARTIFACT_ID, a,
        VERSION, v,
        CLASSIFIER, c,
        FILE_EXTENSION, e));
  }
}
