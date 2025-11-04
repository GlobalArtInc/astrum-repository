
package org.globalart.astrum.mime.internal;

import java.util.Properties;

import org.globalart.astrum.mime.MimeRule;

import com.google.common.base.Joiner;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasProperty;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link AstrumMimeTypes}.
 */
public class AstrumMimeTypesTest
{
  private AstrumMimeTypes underTest = new AstrumMimeTypes();

  private Properties addMimeType(final Properties properties, final String extension, final String... types) {
    properties.setProperty(extension, Joiner.on(",").join(types));
    return properties;
  }

  @Test
  public void unconfigured() {
    assertThat(underTest.getMimeRuleForExtension("test"), is(nullValue()));
  }

  @Test
  public void addMimeType() {
    underTest.initMimeTypes(addMimeType(new Properties(), "test", "application/octet-stream"));
    final MimeRule mimeRule = underTest.getMimeRuleForExtension("test");
    assertThat(mimeRule, is(notNullValue()));
    assertThat(mimeRule, hasProperty("override", is(false)));
    assertThat(mimeRule.getMimetypes(), contains("application/octet-stream"));
  }

  @Test
  public void overrideMimeType() {
    Properties properties = new Properties();
    underTest.initMimeTypes(addMimeType(properties, "override.test", "application/octet-stream"));
    final MimeRule mimeRule = underTest.getMimeRuleForExtension("test");
    assertThat(mimeRule, is(notNullValue()));
    assertThat(mimeRule, hasProperty("override", is(true)));
    assertThat(mimeRule.getMimetypes(), contains("application/octet-stream"));
  }

  @Test
  public void mergeOverrideAndAdditional() {
    Properties types = new Properties();

    addMimeType(types, "override.test", "application/octet-stream");
    addMimeType(types, "test", "text/plain");

    underTest.initMimeTypes(types);
    final MimeRule mimeRule = underTest.getMimeRuleForExtension("test");
    assertThat(mimeRule, is(notNullValue()));
    assertThat(mimeRule, hasProperty("override", is(true)));
    assertThat(mimeRule.getMimetypes(), contains("application/octet-stream", "text/plain"));
  }
}
