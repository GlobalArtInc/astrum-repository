
package org.globalart.astrum.internal.log.overrides.datastore;

import java.io.File;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.datastore.mybatis.ContinuationArrayList;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.LoggerContext;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.rules.TemporaryFolder;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.LoggerFactory;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link DatastoreLoggerOverrides}, focused on the STORAGE-phase {@code doStart()} behavior:
 * applying DB-synced overrides directly to the live logback context and migrating file-only overrides to the DB.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class DatastoreLoggerOverridesTest
{
  @Rule
  public TemporaryFolder tempFolder = new TemporaryFolder();

  @Mock
  private ApplicationDirectories applicationDirectories;

  @Mock
  private LoggingOverridesStore store;

  private File logbackDir;

  private DatastoreLoggerOverrides underTest;

  // loggers whose level this test mutated on the process-global logback context; reset in tearDown
  private final List<String> appliedLoggers = new ArrayList<>();

  @Before
  public void setUp() throws Exception {
    logbackDir = tempFolder.newFolder("etc", "logback");
    when(applicationDirectories.getWorkDirectory("etc/logback")).thenReturn(logbackDir);
    underTest = new DatastoreLoggerOverrides(applicationDirectories, store);
  }

  @After
  public void tearDown() {
    LoggerContext context = loggerContext();
    appliedLoggers.forEach(name -> context.getLogger(name).setLevel(null));
  }

  @Test
  public void doStartAppliesDatastoreOverridesToLogback() throws Exception {
    String loggerName = uniqueLogger("datastore.override.");
    when(store.readRecords()).thenReturn(records(loggerName, LoggerLevel.DEBUG));
    // the override already lives in the DB, so it must not be re-migrated
    when(store.exists(loggerName)).thenReturn(true);

    LoggerContext context = loggerContext();
    assertThat(context.exists(loggerName), nullValue());

    underTest.start();

    // the DB-synced level is pushed directly onto the live logback logger...
    assertThat(context.getLogger(loggerName).getLevel(), is(Level.DEBUG));
    // ...and tracked in the store map
    assertThat(underTest.get(loggerName), is(LoggerLevel.DEBUG));
    verify(store, never()).create(any());
  }

  @Test
  public void doStartWithNoOverridesIsANoOp() throws Exception {
    when(store.readRecords()).thenReturn(records());

    underTest.start();

    assertThat(underTest.iterator().hasNext(), is(false));
    verify(store, never()).create(any());
  }

  @Test
  public void doStartMigratesFileOnlyOverridesToDatastore() throws Exception {
    String loggerName = uniqueLogger("file.only.override.");
    writeLogbackOverridesFile(loggerName, LoggerLevel.WARN);
    // DB has no records; the file-only override must be applied and migrated
    when(store.readRecords()).thenReturn(records());
    when(store.exists(loggerName)).thenReturn(false);

    LoggerContext context = loggerContext();
    assertThat(context.exists(loggerName), nullValue());

    underTest.start();

    // file override is applied to the live logback logger...
    assertThat(context.getLogger(loggerName).getLevel(), is(Level.WARN));
    // ...and migrated into the datastore
    ArgumentCaptor<LoggingOverridesData> captor = ArgumentCaptor.forClass(LoggingOverridesData.class);
    verify(store).create(captor.capture());
    assertThat(captor.getValue().getName(), is(loggerName));
    assertThat(captor.getValue().getLevel(), is(LoggerLevel.WARN.toString()));
  }

  private void writeLogbackOverridesFile(final String name, final LoggerLevel level) throws Exception {
    File file = new File(logbackDir, "logback-overrides.xml");
    String xml = "<?xml version='1.0' encoding='UTF-8'?>\n"
        + "<included>\n"
        + "  <logger name='" + name + "' level='" + level + "'/>\n"
        + "</included>\n";
    Files.write(file.toPath(), xml.getBytes(StandardCharsets.UTF_8));
  }

  private String uniqueLogger(final String prefix) {
    String name = prefix + System.nanoTime();
    appliedLoggers.add(name);
    return name;
  }

  private static LoggerContext loggerContext() {
    return (LoggerContext) LoggerFactory.getILoggerFactory();
  }

  private static Continuation<LoggingOverridesData> records(final Object... nameThenLevel) {
    ContinuationArrayList<LoggingOverridesData> list = new ContinuationArrayList<>();
    for (int i = 0; i < nameThenLevel.length; i += 2) {
      list.add(new LoggingOverridesData((String) nameThenLevel[i], nameThenLevel[i + 1].toString()));
    }
    return list;
  }
}
