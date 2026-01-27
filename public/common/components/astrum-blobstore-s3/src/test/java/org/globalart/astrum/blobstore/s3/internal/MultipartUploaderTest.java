
package org.globalart.astrum.blobstore.s3.internal;

import org.junit.Before;
import org.junit.Test;

import org.globalart.astrum.blobstore.api.BlobStoreException;

import software.amazon.awssdk.core.exception.SdkClientException;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadResponse;
import software.amazon.awssdk.services.s3.model.UploadPartResponse;
import org.mockito.Mock;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.ArrayList;
import java.util.List;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class MultipartUploaderTest
{

  private MultipartUploader multipartUploader;

  @Mock
  private EncryptingS3Client s3;

  @Mock
  private CreateMultipartUploadResponse createMultipartUploadResponse;

  @Before
  public void setUp() {
    when(createMultipartUploadResponse.uploadId()).thenReturn("uploadId");
    multipartUploader = new MultipartUploader(100);
  }

  @Test
  public void testUploadWithMultipartApi() {
    InputStream input = new ByteArrayInputStream(new byte[100]);
    when(s3.createMultipartUpload(any())).thenReturn(createMultipartUploadResponse);
    when(s3.uploadPart(any(), any())).thenReturn(UploadPartResponse.builder().build());

    multipartUploader.upload(s3, "bucketName", "key", input);

    verify(s3).createMultipartUpload(any());
    verify(s3).uploadPart(any(), any());
    verify(s3).completeMultipartUpload(any());
    verify(s3, never()).abortMultipartUpload(any());
  }

  @Test
  public void testUploadAbortsMultipartUploadsOnError() {
    InputStream input = new ByteArrayInputStream(new byte[100]);
    when(s3.createMultipartUpload(any())).thenReturn(createMultipartUploadResponse);
    when(s3.uploadPart(any(), any())).thenThrow(SdkClientException.builder().build());

    assertThrows(BlobStoreException.class, () -> multipartUploader.upload(s3, "bucketName", "key", input));

    verify(s3).createMultipartUpload(any());
    verify(s3).uploadPart(any(), any());
    verify(s3).abortMultipartUpload(any());
  }

  @Test
  public void testReadChunkReadsStreamsInChunks() throws IOException {
    int[] inputSizes = {0, 99, 100, 101, 500};
    int[][] expectedChunkSizes = {
        {0},
        {99, 0},
        {100, 0},
        {100, 1, 0},
        {100, 100, 100, 100, 100, 0}
    };

    for (int i = 0; i < inputSizes.length; i++) {
      InputStream input = new ByteArrayInputStream(new byte[inputSizes[i]]);
      List<InputStream> chunks = new ArrayList<>();
      while (true) {
        try (InputStream chunk = multipartUploader.readChunk(input)) {
          chunks.add(chunk);
          if (chunk.available() == 0) {
            break;
          }
        }
      }
      int[] chunkSizes = chunks.stream().mapToInt(chunk -> {
        try {
          return chunk.available();
        }
        catch (IOException e) {
          throw new UncheckedIOException(e);
        }
      }).toArray();
      assertThat(chunkSizes, is(expectedChunkSizes[i]));
    }
  }

  @Test
  public void testUploadUsesPutObjectForSmallUploads() {
    InputStream input = new ByteArrayInputStream(new byte[50]);
    multipartUploader.upload(s3, "bucketName", "key", input);

    verify(s3).putObject(any(), any());
    verify(s3, never()).createMultipartUpload(any());
  }
}
