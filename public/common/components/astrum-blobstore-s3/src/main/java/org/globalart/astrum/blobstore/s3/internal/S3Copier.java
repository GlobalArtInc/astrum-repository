
package org.globalart.astrum.blobstore.s3.internal;

/**
 * Copies a file in S3.
 *
 * @since 3.15
 */
public interface S3Copier
{

  /**
   * Copies a file in s3.
   */
  void copy(EncryptingS3Client s3, String bucket, String sourcePath, String destinationPath);
}
