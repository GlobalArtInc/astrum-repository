
package org.globalart.astrum.blobstore.s3.internal;

/**
 * Handlers for additional processing needed during s3 bucket operations
 */
public interface BucketOperations
{
  void delete(final String blobstoreName, final String bucketName, final EncryptingS3Client s3Client);
}
