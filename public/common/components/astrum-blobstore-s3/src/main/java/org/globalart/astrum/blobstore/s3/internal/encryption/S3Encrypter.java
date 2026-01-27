
package org.globalart.astrum.blobstore.s3.internal.encryption;

import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;

/**
 * Adds any encryption necessary to S3 requests for AWS SDK 2.x.
 *
 * @since 3.19
 */
public interface S3Encrypter
{
  CreateMultipartUploadRequest.Builder addEncryption(CreateMultipartUploadRequest.Builder requestBuilder);

  PutObjectRequest.Builder addEncryption(PutObjectRequest.Builder requestBuilder);

  CopyObjectRequest.Builder addEncryption(CopyObjectRequest.Builder requestBuilder);
}
