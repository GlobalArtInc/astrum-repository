
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link BlobStoreUpdatedEvent}.
 */
public class BlobStoreUpdatedEventTest
{
  private final BlobStore blobStore = mock(BlobStore.class);

  @Test
  public void getBlobStoreReturnsConstructorArgument() {
    BlobStoreUpdatedEvent underTest = new BlobStoreUpdatedEvent(blobStore);

    assertThat(underTest.getBlobStore(), is(sameInstance(blobStore)));
  }

  @Test
  public void toStringContainsSimpleNameAndBlobStore() {
    BlobStoreUpdatedEvent underTest = new BlobStoreUpdatedEvent(blobStore);

    assertThat(underTest.toString(), is(equalTo("BlobStoreUpdatedEvent{blobStore=" + blobStore + "}")));
  }

  @Test(expected = NullPointerException.class)
  public void nullBlobStoreIsRejected() {
    new BlobStoreUpdatedEvent(null);
  }
}
