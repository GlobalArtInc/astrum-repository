
package org.globalart.astrum.bootstrap.entrypoint;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEdition;
import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSelector;
import org.globalart.astrum.common.app.ApplicationVersion;

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
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link EditionVersionFormatter}
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class EditionVersionFormatterTest
{
  @Mock
  private AstrumEditionSelector astrumEditionSelector;

  @Mock
  private ApplicationVersion applicationVersion;

  @Mock
  private AstrumEdition astrumEdition;

  private Logger logger;

  private ListAppender<ILoggingEvent> logAppender;

  @Before
  public void setUp() {
    // Set up log capture
    logger = (Logger) LoggerFactory.getLogger(EditionVersionFormatter.class);
    logAppender = new ListAppender<>();
    logAppender.start();
    logger.addAppender(logAppender);
  }

  @After
  public void tearDown() {
    logger.detachAppender(logAppender);
  }

  @Test
  public void testFormatEditionAndVersion_withEditionAndVersion() {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

    // Verify
    assertThat(result, is("PRO/3.75.0-01"));
  }

  @Test
  public void testFormatEditionAndVersion_withEditionButNoVersion() {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, null);

    // Verify - should only return edition ID, no slash
    assertThat(result, is("PRO"));
  }

  @Test
  public void testFormatEditionAndVersion_withNullEditionAndVersion() {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(null);
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

    // Verify - should return "unknown" for edition with version
    assertThat(result, is("unknown/3.75.0-01"));

    // Verify warning log
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Unable to determine edition ID"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("level", is(Level.WARN))));
  }

  @Test
  public void testFormatEditionAndVersion_withNullEditionAndNoVersion() {
    // Setup
    when(astrumEditionSelector.getCurrent()).thenReturn(null);

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, null);

    // Verify - should return only "unknown", no slash
    assertThat(result, is("unknown"));

    // Verify warning log
    assertThat(logAppender.list, hasItem(
        hasProperty("formattedMessage", containsString("Unable to determine edition ID"))));
    assertThat(logAppender.list, hasItem(
        hasProperty("level", is(Level.WARN))));
  }

  @Test
  public void testFormatEditionAndVersion_differentEditionIds() {
    // Test with different edition IDs to ensure formatting works correctly
    String[] editionIds = {"PRO", "OSS", "STARTER", "COMMUNITY", "pro-edition", "oss-edition"};

    for (String editionId : editionIds) {
      // Setup
      when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
      when(astrumEdition.getId()).thenReturn(editionId);
      when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

      // Execute
      String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

      // Verify - check each edition ID is properly formatted
      assertThat(result, is(editionId + "/3.75.0-01"));
    }
  }

  @Test
  public void testFormatEditionAndVersion_differentVersions() {
    // Test with different version formats
    String[] versions = {"3.75.0-01", "3.75.0", "4.0.0-SNAPSHOT", "1.0.0-beta", "2.0.0-rc1"};

    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");

    for (String version : versions) {
      // Setup
      when(applicationVersion.getVersion()).thenReturn(version);

      // Execute
      String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

      // Verify - check each version is properly formatted
      assertThat(result, is("PRO/" + version));
    }
  }

  @Test
  public void testFormatEditionAndVersion_onlyLogsWarningOnce() {
    // Setup - null edition
    when(astrumEditionSelector.getCurrent()).thenReturn(null);

    // Execute multiple times
    EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, null);
    int firstCallLogCount = logAppender.list.size();

    logAppender.list.clear();

    EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, null);
    int secondCallLogCount = logAppender.list.size();

    // Verify - warning should be logged each time (not cached)
    assertThat(firstCallLogCount, is(1));
    assertThat(secondCallLogCount, is(1));
  }

  @Test
  public void testFormatEditionAndVersion_handlesEmptyEditionId() {
    // Setup - edition returns empty string
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("");
    when(applicationVersion.getVersion()).thenReturn("3.75.0-01");

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

    // Verify - should use empty string (not "unknown")
    assertThat(result, is("/3.75.0-01"));
  }

  @Test
  public void testFormatEditionAndVersion_handlesEmptyVersion() {
    // Setup - version returns empty string
    when(astrumEditionSelector.getCurrent()).thenReturn(astrumEdition);
    when(astrumEdition.getId()).thenReturn("PRO");
    when(applicationVersion.getVersion()).thenReturn("");

    // Execute
    String result = EditionVersionFormatter.formatEditionAndVersion(astrumEditionSelector, applicationVersion);

    // Verify - should include slash even with empty version
    assertThat(result, is("PRO/"));
  }
}
