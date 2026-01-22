
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link BlobStoreDeletedEvent}.
 */
public class BlobStoreDeletedEventTest
{
  private final BlobStore blobStore = mock(BlobStore.class);

  @Test
  public void getBlobStoreReturnsConstructorArgument() {
    BlobStoreDeletedEvent underTest = new BlobStoreDeletedEvent(blobStore);

    assertThat(underTest.getBlobStore(), is(sameInstance(blobStore)));
  }

  @Test
  public void toStringContainsSimpleNameAndBlobStore() {
    BlobStoreDeletedEvent underTest = new BlobStoreDeletedEvent(blobStore);

    assertThat(underTest.toString(), is(equalTo("BlobStoreDeletedEvent{blobStore=" + blobStore + "}")));
  }

  @Test(expected = NullPointerException.class)
  public void nullBlobStoreIsRejected() {
    new BlobStoreDeletedEvent(null);
  }
}
