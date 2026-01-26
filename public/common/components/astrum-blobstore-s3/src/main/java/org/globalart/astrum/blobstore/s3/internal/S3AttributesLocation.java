
package org.globalart.astrum.blobstore.s3.internal;

import org.globalart.astrum.blobstore.AttributesLocation;

import software.amazon.awssdk.services.s3.model.S3Object;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Location for S3BlobStore attributes files
 *
 * @since 3.15
 */
public class S3AttributesLocation
    implements AttributesLocation
{
  private final String key;

  public S3AttributesLocation(final S3Object s3Object) {
    checkNotNull(s3Object);
    this.key = s3Object.key();
  }

  @Override
  public String getFileName() {
    return key.substring(key.lastIndexOf('/') + 1);
  }

  @Override
  public String getFullPath() {
    return key;
  }
}
