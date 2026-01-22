
package org.globalart.astrum.blobstore.api;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.sameInstance;
import static org.mockito.Mockito.mock;

/**
 * Tests for {@link BlobStoreStartedEvent}.
 */
public class BlobStoreStartedEventTest
{
  private final BlobStore blobStore = mock(BlobStore.class);

  @Test
  public void getBlobStoreReturnsConstructorArgument() {
    BlobStoreStartedEvent underTest = new BlobStoreStartedEvent(blobStore);

    assertThat(underTest.getBlobStore(), is(sameInstance(blobStore)));
  }

  @Test
  public void toStringContainsSimpleNameAndBlobStore() {
    BlobStoreStartedEvent underTest = new BlobStoreStartedEvent(blobStore);

    assertThat(underTest.toString(), is(equalTo("BlobStoreStartedEvent{blobStore=" + blobStore + "}")));
  }

  @Test(expected = NullPointerException.class)
  public void nullBlobStoreIsRejected() {
    new BlobStoreStartedEvent(null);
  }
}
