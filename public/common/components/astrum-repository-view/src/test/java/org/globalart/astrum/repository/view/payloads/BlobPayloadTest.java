
package org.globalart.astrum.repository.view.payloads;

import org.globalart.astrum.blobstore.api.Blob;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

@SuppressWarnings("resource")
@ExtendWith(MockitoExtension.class)
public class BlobPayloadTest
{
  @Mock
  Blob blob;

  @Test
  void testHasRedirect() {
    assertThat(new BlobPayload(blob, null).hasRedirect(), is(false));

    when(blob.hasRedirect()).thenReturn(true);
    assertThat(new BlobPayload(blob, null).hasRedirect(), is(true));
  }
}
