
package org.globalart.astrum.blobstore.api;

import java.io.IOException;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link BlobInputStreamException}.
 */
public class BlobInputStreamExceptionTest
{
  private static final BlobId BLOB_ID = new BlobId("blob-123", null);

  @Test
  public void testCauseAndBlobIdConstructor() {
    IOException cause = new IOException("disk error");
    BlobInputStreamException exception = new BlobInputStreamException(cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, java.io.IOException: disk error"));
  }

  @Test
  public void testCauseConstructorWithNullBlobId() {
    IOException cause = new IOException("disk error");
    BlobInputStreamException exception = new BlobInputStreamException(cause, null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("java.io.IOException: disk error"));
  }

  @Test
  public void testCauseWithNullMessage() {
    RuntimeException cause = new RuntimeException();
    BlobInputStreamException exception = new BlobInputStreamException(cause, null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("java.lang.RuntimeException"));
  }

  @Test
  public void testBlobIdPresentWithCauseLackingMessage() {
    RuntimeException cause = new RuntimeException();
    BlobInputStreamException exception = new BlobInputStreamException(cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, java.lang.RuntimeException"));
  }

  @Test
  public void testInheritsFromBlobStoreException() {
    BlobInputStreamException exception = new BlobInputStreamException(new IOException("oops"), BLOB_ID);

    assertThat(exception instanceof BlobStoreException, is(true));
    assertThat(exception instanceof RuntimeException, is(true));
  }
}
