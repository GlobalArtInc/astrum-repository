
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

/**
 * Tests for {@link BlobStoreConnectionException}.
 */
public class BlobStoreConnectionExceptionTest
{
  @Test
  public void testConstructorAndMessage() {
    String message = "Unable to connect to the blob store source";
    BlobStoreConnectionException exception = new BlobStoreConnectionException(message);

    assertThat(exception.getMessage(), is(message));
    assertThat(exception.getCause(), is(nullValue()));
  }

  @Test
  public void testConstructorWithNullMessage() {
    BlobStoreConnectionException exception = new BlobStoreConnectionException(null);

    assertThat(exception.getMessage(), is(nullValue()));
    assertThat(exception.getCause(), is(nullValue()));
  }

  @Test
  public void testConstructorWithEmptyMessage() {
    BlobStoreConnectionException exception = new BlobStoreConnectionException("");

    assertThat(exception.getMessage(), is(""));
    assertThat(exception.getCause(), is(nullValue()));
  }

  @Test
  public void testInheritsFromRuntimeException() {
    BlobStoreConnectionException exception = new BlobStoreConnectionException("boom");

    assertThat(exception instanceof RuntimeException, is(true));
  }
}
