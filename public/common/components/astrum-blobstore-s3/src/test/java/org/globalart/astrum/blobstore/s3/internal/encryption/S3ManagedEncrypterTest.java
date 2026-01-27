
package org.globalart.astrum.blobstore.s3.internal.encryption;

import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.ServerSideEncryption;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class S3ManagedEncrypterTest
{

  @Mock
  private CreateMultipartUploadRequest.Builder createMultipartUploadRequestBuilder;

  @Mock
  private PutObjectRequest.Builder putObjectRequestBuilder;

  @Mock
  private CopyObjectRequest.Builder copyObjectRequestBuilder;

  private final S3ManagedEncrypter encrypter = new S3ManagedEncrypter();

  @Test
  public void testS3ManagedServerSideEncWorksForCreateMultipartUploadRequest() {
    when(createMultipartUploadRequestBuilder.serverSideEncryption(any(ServerSideEncryption.class)))
        .thenReturn(createMultipartUploadRequestBuilder);

    encrypter.addEncryption(createMultipartUploadRequestBuilder);

    verify(createMultipartUploadRequestBuilder).serverSideEncryption(ServerSideEncryption.AES256);
  }

  @Test
  public void testS3ManagedServerSideEncWorksForPutObjectRequest() {
    when(putObjectRequestBuilder.serverSideEncryption(any(ServerSideEncryption.class)))
        .thenReturn(putObjectRequestBuilder);

    encrypter.addEncryption(putObjectRequestBuilder);

    verify(putObjectRequestBuilder).serverSideEncryption(ServerSideEncryption.AES256);
  }

  @Test
  public void testS3ManagedServerSideEncWorksForCopyObjectRequest() {
    when(copyObjectRequestBuilder.serverSideEncryption(any(ServerSideEncryption.class)))
        .thenReturn(copyObjectRequestBuilder);

    encrypter.addEncryption(copyObjectRequestBuilder);

    verify(copyObjectRequestBuilder).serverSideEncryption(ServerSideEncryption.AES256);
  }
}
