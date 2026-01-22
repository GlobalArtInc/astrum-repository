
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link BlobStoreStoppedEvent}.
 */
public class BlobStoreStoppedEventTest
{
  private final BlobStore blobStore = mock(BlobStore.class);

  @Test
  public void getBlobStoreReturnsConstructorArgument() {
    BlobStoreStoppedEvent underTest = new BlobStoreStoppedEvent(blobStore);

    assertThat(underTest.getBlobStore(), is(sameInstance(blobStore)));
  }

  @Test
  public void toStringContainsSimpleNameAndBlobStore() {
    BlobStoreStoppedEvent underTest = new BlobStoreStoppedEvent(blobStore);

    assertThat(underTest.toString(), is(equalTo("BlobStoreStoppedEvent{blobStore=" + blobStore + "}")));
  }

  @Test(expected = NullPointerException.class)
  public void nullBlobStoreIsRejected() {
    new BlobStoreStoppedEvent(null);
  }
}
