
package org.globalart.astrum.validation.constraint;

import java.util.Arrays;
import java.util.Collection;

import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameter;
import org.junit.runners.Parameterized.Parameters;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;

@RunWith(Parameterized.class)
public class UrlStringValidatorTest

{
  private final UrlStringValidator underTest = new UrlStringValidator();

  @Parameter()
  public String url;

  @Parameter(1)
  public boolean valid;

  @Parameters(name = "{index}: ''{0}'' should be valid: {1}")
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {"http://example", true},
        {"www", false}
    });
  }

  @Test
  public void shouldValidateUrlString() throws Exception {
    TestDatum datum = new TestDatum(url);
    underTest.initialize(getUrlStringAnnotation(datum));
    assertThat(
        String.format("'%s' should be %s", url, valid ? "valid" : "invalid"),
        underTest.isValid(datum.getUrl(), null), is(valid));
  }

  private UrlString getUrlStringAnnotation(final Object obj) throws NoSuchFieldException {
    return obj.getClass().getDeclaredField("url").getAnnotation(UrlString.class);
  }

  public static class TestDatum
  {
    @UrlString
    private final String url;

    public TestDatum(final String url) {
      this.url = url;
    }

    public String getUrl() {
      return url;
    }
  }
}
