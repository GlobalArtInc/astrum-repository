
package org.globalart.astrum.blobstore.s3.internal;

import org.junit.Before;
import org.junit.Test;

import org.globalart.astrum.blobstore.api.BlobStoreException;

import software.amazon.awssdk.core.exception.SdkClientException;
import software.amazon.awssdk.services.s3.model.CopyPartResult;
import software.amazon.awssdk.services.s3.model.UploadPartCopyResponse;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadResponse;
import software.amazon.awssdk.services.s3.model.HeadObjectResponse;

import org.mockito.Mock;

import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MultipartCopierTest
{

  private MultipartCopier multipartCopier;

  @Mock
  private EncryptingS3Client s3;

  @Mock
  private CreateMultipartUploadResponse createMultipartUploadResponse;

  @Before
  public void setUp() {
    multipartCopier = new MultipartCopier(100);
  }

  @Test
  public void testCopyWithMultipartApi() {
    when(s3.createMultipartUpload(any())).thenReturn(createMultipartUploadResponse);
    when(s3.getObjectMetadata("bucketName", "source")).thenReturn(HeadObjectResponse.builder()
        .contentLength(101L)
        .build());
    when(s3.uploadPartCopy(any())).thenReturn(UploadPartCopyResponse.builder()
        .copyPartResult(CopyPartResult.builder()
            .eTag("test-etag")
            .build())
        .build());

    multipartCopier.copy(s3, "bucketName", "source", "destination");

    verify(s3).createMultipartUpload(any());
    verify(s3).getObjectMetadata("bucketName", "source");
    verify(s3, times(2)).uploadPartCopy(any());
    verify(s3).completeMultipartUpload(any());
    verify(s3, never()).abortMultipartUpload(any());
  }

  @Test
  public void testCopyAbortsMultipartOnError() {
    when(createMultipartUploadResponse.uploadId()).thenReturn("uploadId");
    when(s3.createMultipartUpload(any())).thenReturn(createMultipartUploadResponse);
    when(s3.getObjectMetadata("bucketName", "source")).thenReturn(HeadObjectResponse.builder()
        .contentLength(101L)
        .build());
    when(s3.uploadPartCopy(any())).thenThrow(SdkClientException.builder().build());

    assertThrows(BlobStoreException.class, () -> multipartCopier.copy(s3, "bucketName", "source", "destination"));

    verify(s3).createMultipartUpload(any());
    verify(s3).getObjectMetadata("bucketName", "source");
    verify(s3).uploadPartCopy(any());
    verify(s3).abortMultipartUpload(any());
  }

  @Test
  public void testCopySplitsParts() {
    when(s3.createMultipartUpload(any())).thenReturn(createMultipartUploadResponse);
    when(s3.getObjectMetadata("bucketName", "source")).thenReturn(HeadObjectResponse.builder()
        .contentLength(345L)
        .build());
    when(s3.uploadPartCopy(any())).thenReturn(UploadPartCopyResponse.builder()
        .copyPartResult(CopyPartResult.builder()
            .eTag("test-etag")
            .build())
        .build());

    multipartCopier.copy(s3, "bucketName", "source", "destination");

    verify(s3).createMultipartUpload(any());
    verify(s3).getObjectMetadata("bucketName", "source");
    verify(s3, times(4)).uploadPartCopy(any());
    verify(s3).completeMultipartUpload(any());
    verify(s3, never()).abortMultipartUpload(any());
  }

  @Test
  public void testCopyUsesCopyObjectForSmallCopies() {
    when(s3.getObjectMetadata("bucketName", "source")).thenReturn(HeadObjectResponse.builder()
        .contentLength(99L)
        .build());

    multipartCopier.copy(s3, "bucketName", "source", "destination");

    verify(s3).getObjectMetadata("bucketName", "source");
    verify(s3).copyObject(anyString(), anyString(), anyString(), anyString());
    verify(s3, never()).createMultipartUpload(any());
  }
}
