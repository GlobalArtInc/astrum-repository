
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
public class UriStringValidatorTest

{
  private final UriStringValidator underTest = new UriStringValidator();

  @Parameter()
  public String uri;

  @Parameter(1)
  public boolean valid;

  @Parameters(name = "{index}: ''{0}'' should be valid: {1}")
  public static Collection<Object[]> data() {
    return Arrays.asList(new Object[][]{
        {"http://example", true},
        {"www", true},
        {"<>invaliduri", false}
    });
  }

  @Test
  public void shouldValidateUriString() throws Exception {
    TestDatum datum = new TestDatum(uri);
    underTest.initialize(getUriStringAnnotation(datum));
    assertThat(
        String.format("'%s' should be %s", uri, valid ? "valid" : "invalid"),
        underTest.isValid(datum.getUri(), null), is(valid));
  }

  private UriString getUriStringAnnotation(final Object obj) throws NoSuchFieldException {
    return obj.getClass().getDeclaredField("uri").getAnnotation(UriString.class);
  }

  public static class TestDatum
  {
    @UriString
    private final String uri;

    public TestDatum(final String uri) {
      this.uri = uri;
    }

    public String getUri() {
      return uri;
    }
  }
}
