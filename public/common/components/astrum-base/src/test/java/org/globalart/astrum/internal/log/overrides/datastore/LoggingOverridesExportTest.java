
package org.globalart.astrum.internal.log.overrides.datastore;

import java.io.File;
import java.io.IOException;
import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.stream.Collectors;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import com.google.common.collect.ForwardingCollection;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization {@link LoggingOverridesData}
 * by {@link LoggingOverridesExport}
 */
public class LoggingOverridesExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File jsonFile;

  @Before
  public void setup() throws IOException {
    jsonFile = File.createTempFile("LoggingOverrides", ".json");
  }

  @After
  public void tearDown() {
    jsonFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    List<LoggingOverridesData> configurationData = Arrays.asList(
        createLoggingOverride("com.example.foo", LoggerLevel.DEBUG),
        createLoggingOverride("com.example.bar", LoggerLevel.TRACE));

    LoggingOverridesStore store = mock(LoggingOverridesStore.class);
    when(store.readRecords()).thenReturn(new ContinuationMock<>(configurationData));

    LoggingOverridesExport exporter = new LoggingOverridesExport(store);
    exporter.export(jsonFile);
    List<LoggingOverridesData> importedData = jsonExporter.importFromJson(jsonFile, LoggingOverridesData.class);

    assertThat(importedData.size(), is(2));
    assertThat(importedData.stream().map(LoggingOverridesData::getName).collect(Collectors.toList()),
        hasItem("com.example.foo"));
    assertThat(importedData.stream().map(LoggingOverridesData::getName).collect(Collectors.toList()),
        hasItem("com.example.bar"));
    assertThat(importedData.stream().map(LoggingOverridesData::getLevel).collect(Collectors.toList()),
        hasItem(LoggerLevel.DEBUG.toString()));
    assertThat(importedData.stream().map(LoggingOverridesData::getLevel).collect(Collectors.toList()),
        hasItem(LoggerLevel.TRACE.toString()));
  }

  private LoggingOverridesData createLoggingOverride(final String name, final LoggerLevel level) {
    return new LoggingOverridesData(name, level.toString());
  }

  /**
   * Simple test continuation implementation
   */
  private static class ContinuationMock<E>
      extends ForwardingCollection<E>
      implements Continuation<E>
  {
    private final Collection<E> collection;

    public ContinuationMock(final Collection<E> collection) {
      this.collection = collection;
    }

    @Override
    protected Collection<E> delegate() {
      return collection;
    }

    @Override
    public String nextContinuationToken() {
      return null;
    }
  }
}
