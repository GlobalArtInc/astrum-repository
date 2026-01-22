
package org.globalart.astrum.blobstore.api;

import java.net.URL;
import java.util.Optional;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.CALLS_REAL_METHODS;
import static org.mockito.Mockito.mock;

/**
 * Tests for the default methods of {@link Blob}.
 */
public class BlobTest
{
  private Blob underTest;

  @Before
  public void setUp() {
    underTest = mock(Blob.class, CALLS_REAL_METHODS);
  }

  @Test
  public void getRedirectUrlReturnsEmptyByDefault() {
    Optional<URL> result = underTest.getRedirectUrl("download", "name.txt", "text/plain");

    assertThat(result.isPresent(), is(false));
  }

  @Test
  public void getRedirectUrlReturnsEmptyWithNullContentType() {
    Optional<URL> result = underTest.getRedirectUrl("download", "name.txt", null);

    assertThat(result.isPresent(), is(false));
  }

  @Test
  public void getRedirectUrlReturnsEmptyRegardlessOfArguments() {
    // the default body ignores every argument, so even null action/name must not throw and still yields empty
    Optional<URL> result = underTest.getRedirectUrl(null, null, null);

    assertThat(result.isPresent(), is(false));
  }
}
