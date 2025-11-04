
package org.globalart.astrum.mime;

import org.hamcrest.Matcher;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link RegexpMimeRulesSource}.
 */
public class RegexpMimeRulesSourceTest
{
  @Test
  public void testRegectMimeRulesSourceTest() {
    final RegexpMimeRulesSource underTest = new RegexpMimeRulesSource();

    underTest.addRule(".*\\.foo\\z", "foo/bar");
    underTest.addRule(".*\\.pom\\z", "application/x-pom");
    underTest.addRule("(.*/maven-metadata.xml\\z)|(maven-metadata.xml\\z)", "application/x-maven-metadata");

    // "more specific" one
    underTest.addRule("\\A/atom-service/.*\\.xml\\z", "application/atom+xml");

    // and now the "general one"
    underTest.addRule(".*\\.xml\\z", "application/xml");

    assertValues(underTest, "/some/repo/path/content.foo", equalTo("foo/bar"));
    assertValues(underTest, "/some/repo/path/content.foo.bar", nullValue());
    assertValues(underTest, "/log4j/log4j/1.2.12/log4j-1.2.12.pom", equalTo("application/x-pom"));
    assertValues(underTest, "maven-metadata.xml", equalTo("application/x-maven-metadata"));
    assertValues(underTest, "/maven-metadata.xml", equalTo("application/x-maven-metadata"));
    assertValues(underTest, "/org/globalart/astrum/maven-metadata.xml", equalTo("application/x-maven-metadata"));
    assertValues(underTest, "/org/globalart/astrum/maven-metadata.xml.bar", nullValue());
    assertValues(underTest, "/org/globalart/astrum/maven-metadata.xml/tricky/path.pom", equalTo("application/x-pom"));

    assertValues(underTest, "/org/globalart/astrum/maven-metadata1.xml", equalTo("application/xml"));
    assertValues(underTest, "/atom-service//org/globalart/astrum/maven-metadata1.xml", equalTo("application/atom+xml"));
  }

  private void assertValues(
      final MimeRulesSource mimeRulesSource,
      final String path,
      final Matcher<? super String> matcher)
  {
    MimeRule astrumMimeType = mimeRulesSource.getRuleForName(path);
    if (matcher.getClass().equals(nullValue().getClass())) {
      assertThat(astrumMimeType, nullValue());
      return;
    }
    assertThat(astrumMimeType, notNullValue());
    assertThat(astrumMimeType.getMimetypes(), hasSize(1));
    assertThat(astrumMimeType.getMimetypes().get(0), matcher);
  }
}
