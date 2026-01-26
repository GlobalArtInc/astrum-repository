
package org.globalart.astrum.blobstore.internal;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.softdeleted.SoftDeletedBlobsStore;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SoftDeletedBlobIndexImplTest
{
  @Mock
  BlobStore blobStore;

  @Mock
  BlobStoreConfiguration blobStoreConfiguration;

  @Mock
  SoftDeletedBlobsStore softDeletedBlobsStore;

  @InjectMocks
  SoftDeletedBlobIndexImpl underTest;

  @BeforeEach
  void setup() {
    when(blobStore.getBlobStoreConfiguration()).thenReturn(blobStoreConfiguration);
    when(blobStoreConfiguration.getName()).thenReturn("a-blobstore");
  }

  @Test
  void testInit() {
    underTest.init(blobStore);

    // verify blobstore name is set
    underTest.createRecord(mock(BlobId.class));
    verify(softDeletedBlobsStore).createRecord(any(), eq("a-blobstore"));

    // this should not trigger an exception
    underTest.init(blobStore);
  }
}
