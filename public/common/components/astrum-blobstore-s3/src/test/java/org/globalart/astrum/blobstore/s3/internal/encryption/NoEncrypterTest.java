
package org.globalart.astrum.blobstore.s3.internal.encryption;

import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class NoEncrypterTest
{

  private final NoEncrypter noEncrypter = new NoEncrypter();

  @Mock
  private CreateMultipartUploadRequest.Builder createMultipartUploadRequestBuilder;

  @Mock
  private PutObjectRequest.Builder putObjectRequestBuilder;

  @Mock
  private CopyObjectRequest.Builder copyObjectRequestBuilder;

  @Test
  public void testNoEncrypterDoesNothingToInitiateMultipartUploadRequest() {
    noEncrypter.addEncryption(createMultipartUploadRequestBuilder);
    verifyNoInteractions(createMultipartUploadRequestBuilder);
  }

  @Test
  public void testNoEncrypterDoesNothingToAbstractPutObjectRequest() {
    noEncrypter.addEncryption(putObjectRequestBuilder);
    verifyNoInteractions(putObjectRequestBuilder);
  }

  @Test
  public void testNoEncrypterDoesNothingToCopyObjectRequest() {
    noEncrypter.addEncryption(copyObjectRequestBuilder);
    verifyNoInteractions(copyObjectRequestBuilder);
  }
}
