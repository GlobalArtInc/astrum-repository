
package org.globalart.astrum.api.rest.common.blobstore.s3;

/**
 * Exception thrown when an attempt is made to enable S3 pre signed url via REST for a non-pro version
 */
public class PreSignedUrlNotAllowedException
    extends IllegalArgumentException
{
  public PreSignedUrlNotAllowedException(final String message) {
    super(message);
  }
}
