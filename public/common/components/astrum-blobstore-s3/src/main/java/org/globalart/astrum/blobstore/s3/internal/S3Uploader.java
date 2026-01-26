
package org.globalart.astrum.blobstore.s3.internal;

import java.io.InputStream;

/**
 * Uploads a file to S3.
 *
 * @since 3.12
 */
public interface S3Uploader
{

  /**
   * Upload a file to s3.
   */
  void upload(EncryptingS3Client s3, String bucket, String key, InputStream contents);
}
