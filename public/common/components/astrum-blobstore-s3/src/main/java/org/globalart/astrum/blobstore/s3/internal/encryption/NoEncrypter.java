
package org.globalart.astrum.blobstore.s3.internal.encryption;

import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Adds no encryption to a S3 request.
 *
 * @since 3.19
 */
@Component
@Qualifier(NoEncrypter.ID)
public class NoEncrypter
    implements S3Encrypter
{
  public static final String ID = "none";

  public static final String NAME = "None";

  public static final NoEncrypter INSTANCE = new NoEncrypter();

  @Override
  public CreateMultipartUploadRequest.Builder addEncryption(final CreateMultipartUploadRequest.Builder requestBuilder) {
    return requestBuilder;
  }

  @Override
  public PutObjectRequest.Builder addEncryption(final PutObjectRequest.Builder requestBuilder) {
    return requestBuilder;
  }

  @Override
  public CopyObjectRequest.Builder addEncryption(final CopyObjectRequest.Builder requestBuilder) {
    return requestBuilder;
  }
}
