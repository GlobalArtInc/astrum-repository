
package org.globalart.astrum.bootstrap.entrypoint;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEdition;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.bootstrap.entrypoint.jvm.ShutdownDelegate;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.ManagedLifecycle.Phase;
import org.globalart.astrum.extender.AstrumLifecycleManager;

import ch.qos.logback.classic.Level;
import ch.qos.logback.classic.Logger;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.read.ListAppender;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;
import org.slf4j.LoggerFactory;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.hasProperty;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link AstrumLifecycleLauncher}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AstrumLifecycleLauncherTest
{
  @Mock
  private AstrumEditionSelector astrumEditionSelector;

  @Mock
  private ShutdownDelegate shutdownDelegate;

  @Mock
  private ApplicationVersion applicationVersion;

  @Mock
  private AstrumEdition astrumEdition;

  @Mock
  private AstrumLifecycleManager astrumLifecycleManager;

  private AstrumLifecycleLauncher launcher;

  private Logger logger;

  private Logger formatterLogger;

  private ListAppender<ILoggingEvent> logAppender;

  @Before
  public void setUp() {
    // Set up log capture for AstrumLifecycleLauncher
    logger = (Logger) LoggerFactory.getLogger(AstrumLifecycleLauncher.class);
    logAppender = new ListAppender<>();
    logAppender.start();
    logger.addAppender(logAppender);

    // Set up log capture for EditionVersionFormatter (for warning messages)
    formatterLogger = (Logger) LoggerFactory.getLogger(EditionVersionFormatter.class);
    formatterLogger.addAppender(logAppender);

    launcher = new AstrumLifecycleLauncher(astrumEditionSelector, shutdownDelegate, applicationVersion, null);
  }

  @After
  public void tearDown() {
    logger.detachAppender(logAppender);
    formatterLogger.detachAppender(logAppender);
  }

  @Test
  public void testStop_withEditionAndVersion() throws Exception {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Execute
    launcher.stop();

    // Verify - check log message contains edition and version
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("PRO/3.75.0-01"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Stopping Astrum"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("level", is(Level.INFO))));

    // astrumLifecycleManager is null, so it should not be called
    verifyNoInteractions(astrumLifecycleManager);
  }

  @Test
  public void testStop_withEditionButNoVersion() throws Exception {
    // Setup - no application version
    launcher = new AstrumLifecycleLauncher(astrumEditionSelector, shutdownDelegate, null, null);
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");

    // Execute
    launcher.stop();

    // Verify - check log message contains only edition (no version)
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("PRO"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Stopping Astrum"))));
    // Should NOT contain a slash followed by version
    for (ILoggingEvent event : logAppender.list) {
      if (event.getFormattedMessage().contains("Stopping Astrum")) {
        assertThat(event.getFormattedMessage().contains("PRO/"), is(false));
      }
    }
  }

  @Test
  public void testStop_withNullEditionAndVersion() throws Exception {
    // Setup - null edition but version present
    when(astrumEditionSelector.getCurrent()).thenReturn(null);
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Execute
    launcher.stop();

    // Verify - check log message contains "unknown" for edition with version
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("unknown/3.75.0-01"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Stopping Astrum"))));
    // Verify warning log about unable to determine edition
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Unable to determine edition ID"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("level", is(Level.WARN))));
  }

  @Test
  public void testStop_withNullEditionAndNoVersion() throws Exception {
    // Setup - null edition and no version
    launcher = new AstrumLifecycleLauncher(astrumEditionSelector, shutdownDelegate, null, null);
    when(astrumEditionSelector.getCurrent()).thenReturn(null);

    // Execute
    launcher.stop();

    // Verify - check log message contains only "unknown"
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("unknown"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Stopping Astrum"))));
    // Verify warning log
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Unable to determine edition ID"))));
    // Should NOT contain a slash
    for (ILoggingEvent event : logAppender.list) {
      if (event.getFormattedMessage().contains("Stopping Astrum")) {
        assertThat(event.getFormattedMessage().contains("unknown/"), is(false));
      }
    }
  }

  @Test
  public void testStop_withAstrumLifecycleManager() throws Exception {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("OSS");
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Use reflection to set the astrumLifecycleManager (normally set via onAstrumScanComplete)
    java.lang.reflect.Field field = AstrumLifecycleLauncher.class.getDeclaredField("astrumLifecycleManager");
    field.setAccessible(true);
    field.set(launcher, astrumLifecycleManager);

    // Execute
    launcher.stop();

    // Verify - astrumLifecycleManager should be called to move to OFF phase
    verify(astrumLifecycleManager).to(Phase.OFF);

    // Verify log message
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("OSS/3.75.0-01"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Stopping Astrum"))));
  }

  @Test
  public void testFormatEditionAndVersion_differentEditionIds() throws Exception {
    // Test with different edition IDs to ensure formatting works correctly
    String[] editionIds = {"PRO", "OSS", "STARTER", "COMMUNITY"};

    for (String editionId : editionIds) {
      logAppender.list.clear(); // Clear logs between tests

      // Setup
      when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
      when(astrumEdition.getId()).thenReturn(editionId);
      when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

      // Execute
      launcher.stop();

      // Verify - check each edition ID is properly formatted
      assertThat(logAppender.list, hasItem(
          hasProperty("formattedMessage", containsString(editionId + "/3.75.0-01"))));
    }
  }

  @Test
  public void testConstructor_withApplicationVersion() {
    // Test constructor accepts ApplicationVersion
    AstrumLifecycleLauncher launcherWithVersion = new AstrumLifecycleLauncher(
        astrumEditionSelector,
        shutdownDelegate,
        applicationVersion,
        null);

    // Verify no exceptions thrown - constructor handles nullable ApplicationVersion
    assertThat(launcherWithVersion != null, is(true));
  }

  @Test
  public void testConstructor_withoutApplicationVersion() {
    // Test constructor accepts null ApplicationVersion
    AstrumLifecycleLauncher launcherWithoutVersion = new AstrumLifecycleLauncher(
        astrumEditionSelector,
        shutdownDelegate,
        null,
        null);

    // Verify no exceptions thrown - constructor handles nullable ApplicationVersion
    assertThat(launcherWithoutVersion != null, is(true));
  }
}
