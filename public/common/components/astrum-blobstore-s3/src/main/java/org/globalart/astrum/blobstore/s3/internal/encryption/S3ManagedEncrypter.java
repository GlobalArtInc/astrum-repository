
package org.globalart.astrum.blobstore.s3.internal.encryption;

import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.s3.model.ServerSideEncryption;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Adds S3 managed encryption to S3 requests using AWS SDK 2.x.
 * 
 * @since 3.19
 */
@Component
@Qualifier(S3ManagedEncrypter.ID)
public class S3ManagedEncrypter
    implements S3Encrypter
{
  public static final String ID = "s3ManagedEncryption";

  public static final String NAME = "S3 Managed Encryption";

  @Override
  public CreateMultipartUploadRequest.Builder addEncryption(final CreateMultipartUploadRequest.Builder requestBuilder) {
    return requestBuilder.serverSideEncryption(ServerSideEncryption.AES256);
  }

  @Override
  public PutObjectRequest.Builder addEncryption(final PutObjectRequest.Builder requestBuilder) {
    return requestBuilder.serverSideEncryption(ServerSideEncryption.AES256);
  }

  @Override
  public CopyObjectRequest.Builder addEncryption(final CopyObjectRequest.Builder requestBuilder) {
    return requestBuilder.serverSideEncryption(ServerSideEncryption.AES256);
  }
}
