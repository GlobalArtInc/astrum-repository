
package org.globalart.astrum.blobstore.api;

import java.io.IOException;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

/**
 * Tests for {@link BlobStoreException}.
 */
public class BlobStoreExceptionTest
{
  private static final BlobId BLOB_ID = new BlobId("blob-123", null);

  @Test
  public void testMessageAndBlobIdConstructor() {
    BlobStoreException exception = new BlobStoreException("boom", BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(nullValue()));
    assertThat(exception.getMessage(), is("BlobId: blob-123, boom"));
  }

  @Test
  public void testMessageConstructorWithNullBlobId() {
    BlobStoreException exception = new BlobStoreException("boom", (BlobId) null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(nullValue()));
    assertThat(exception.getMessage(), is("boom"));
  }

  @Test
  public void testMessageConstructorWithNullMessageAndBlobId() {
    BlobStoreException exception = new BlobStoreException((String) null, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(nullValue()));
    assertThat(exception.getMessage(), is("BlobId: blob-123"));
  }

  @Test
  public void testMessageCauseAndBlobIdConstructor() {
    IOException cause = new IOException("disk full");
    BlobStoreException exception = new BlobStoreException("failed", cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, failed, Cause: disk full"));
  }

  @Test
  public void testMessageCauseConstructorWithNullBlobId() {
    RuntimeException cause = new RuntimeException("inner");
    BlobStoreException exception = new BlobStoreException("failed", cause, null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("failed, Cause: inner"));
  }

  @Test
  public void testMessageCauseConstructorWithCauseHavingNullMessage() {
    RuntimeException cause = new RuntimeException();
    BlobStoreException exception = new BlobStoreException("failed", cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, failed"));
  }

  @Test
  public void testCauseAndBlobIdConstructor() {
    IOException cause = new IOException("io error");
    BlobStoreException exception = new BlobStoreException(cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, java.io.IOException: io error"));
  }

  @Test
  public void testCauseConstructorWithNullBlobId() {
    IllegalStateException cause = new IllegalStateException("bad state");
    BlobStoreException exception = new BlobStoreException(cause, null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("java.lang.IllegalStateException: bad state"));
  }

  @Test
  public void testCauseConstructorWithNullCauseAndNullBlobId() {
    BlobStoreException exception = new BlobStoreException((Throwable) null, (BlobId) null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(nullValue()));
    assertThat(exception.getMessage(), is(nullValue()));
  }

  @Test
  public void testMessageConstructorWithNullMessageAndNullBlobId() {
    BlobStoreException exception = new BlobStoreException((String) null, (BlobId) null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(nullValue()));
    assertThat(exception.getMessage(), is(nullValue()));
  }

  @Test
  public void testMessageCauseConstructorWithNullMessageAndBlobId() {
    IOException cause = new IOException("disk full");
    BlobStoreException exception = new BlobStoreException((String) null, cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, Cause: disk full"));
  }

  @Test
  public void testMessageCauseConstructorWithCauseHavingNullMessageAndNullBlobId() {
    RuntimeException cause = new RuntimeException();
    BlobStoreException exception = new BlobStoreException("failed", cause, null);

    assertThat(exception.getBlobId(), is(nullValue()));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("failed"));
  }

  @Test
  public void testCauseConstructorWithCauseHavingNullMessage() {
    RuntimeException cause = new RuntimeException();
    BlobStoreException exception = new BlobStoreException(cause, BLOB_ID);

    assertThat(exception.getBlobId(), is(sameInstance(BLOB_ID)));
    assertThat(exception.getCause(), is(sameInstance((Throwable) cause)));
    assertThat(exception.getMessage(), is("BlobId: blob-123, java.lang.RuntimeException"));
  }

  @Test
  public void testGetMessageIsStableAcrossRepeatedCalls() {
    BlobStoreException exception = new BlobStoreException("failed", new IOException("disk full"), BLOB_ID);

    String first = exception.getMessage();
    String second = exception.getMessage();

    assertThat(first, is("BlobId: blob-123, failed, Cause: disk full"));
    assertThat(second, is(first));
  }

  @Test
  public void testInheritsFromRuntimeException() {
    BlobStoreException exception = new BlobStoreException("boom", BLOB_ID);

    assertThat(exception instanceof RuntimeException, is(true));
  }
}
