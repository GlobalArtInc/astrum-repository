
package org.globalart.astrum.blobstore.s3.internal.encryption;

import java.util.Optional;

import software.amazon.awssdk.services.s3.model.CopyObjectRequest;
import software.amazon.awssdk.services.s3.model.CreateMultipartUploadRequest;
import software.amazon.awssdk.services.s3.model.PutObjectRequest;
import software.amazon.awssdk.services.s3.model.ServerSideEncryption;
import com.google.common.annotations.VisibleForTesting;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.empty;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Adds KMS encryption to S3 requests.
 * The keyID is optional and in the params
 *
 * @since 3.19
 */
@Component
@Qualifier(KMSEncrypter.ID)
public class KMSEncrypter
    implements S3Encrypter
{
  public static final String ID = "kmsManagedEncryption";

  public static final String NAME = "KMS Managed Encryption";

  private final String kmsKeyId;

  @VisibleForTesting
  String getKmsKeyId() {
    return kmsKeyId;
  }

  public KMSEncrypter() {
    this(empty());
  }

  public KMSEncrypter(final Optional<String> kmsId) {
    this.kmsKeyId = checkNotNull(kmsId)
        .map(String::trim)
        .filter(id -> !id.isEmpty())
        .orElse(null);
  }

  @Override
  public CreateMultipartUploadRequest.Builder addEncryption(final CreateMultipartUploadRequest.Builder requestBuilder) {
    requestBuilder.serverSideEncryption(ServerSideEncryption.AWS_KMS);
    if (kmsKeyId != null) {
      requestBuilder.ssekmsKeyId(kmsKeyId);
    }
    return requestBuilder;
  }

  @Override
  public PutObjectRequest.Builder addEncryption(final PutObjectRequest.Builder requestBuilder) {
    requestBuilder.serverSideEncryption(ServerSideEncryption.AWS_KMS);
    if (kmsKeyId != null) {
      requestBuilder.ssekmsKeyId(kmsKeyId);
    }
    return requestBuilder;
  }

  @Override
  public CopyObjectRequest.Builder addEncryption(final CopyObjectRequest.Builder requestBuilder) {
    requestBuilder.serverSideEncryption(ServerSideEncryption.AWS_KMS);
    if (kmsKeyId != null) {
      requestBuilder.ssekmsKeyId(kmsKeyId);
    }
    return requestBuilder;
  }
}
