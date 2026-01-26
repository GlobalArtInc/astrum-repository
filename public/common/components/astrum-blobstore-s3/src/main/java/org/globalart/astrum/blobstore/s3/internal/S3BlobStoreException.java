
package org.globalart.astrum.blobstore.s3.internal;

import java.util.Map;
import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobStoreException;

import software.amazon.awssdk.services.s3.model.S3Exception;
import com.google.common.collect.ImmutableMap;

import static java.lang.String.format;

/**
 * A {@link BlobStoreException} specific to the S3 implementation
 *
 * @since 3.19
 */
public class S3BlobStoreException
    extends BlobStoreException
{
  public static final String DEFAULT_MESSAGE = "An unexpected S3 error occurred. Check the logs for more details.";

  public static final String INSUFFICIENT_PERM_CREATE_BUCKET_ERR_MSG = "Insufficient permissions to create bucket.";

  public static final String UNEXPECTED_ERR = "An unexpected error occurred %s. Check the logs for more details.";

  public static final String BUCKET_OWNERSHIP_ERR_MSG = "Bucket exists but is not owned by you.";

  // If you have the correct permissions, but you're not using an identity
  // that belongs to the bucket owner's account, Amazon S3 returns a 405 Method Not Allowed error.
  // https://docs.aws.amazon.com/cli/latest/reference/s3api/get-bucket-policy.html#description
  public static final String INVALID_IDENTITY_ERR_MSG =
      "The identity used does not belong to the bucket owner's account.";

  public static final String ACCESS_DENIED_CODE = "AccessDenied";

  public static final String METHOD_NOT_ALLOWED_CODE = "MethodNotAllowed";

  public static final String INVALID_ACCESS_KEY_ID_CODE = "InvalidAccessKeyId";

  public static final String SIGNATURE_DOES_NOT_MATCH_CODE = "SignatureDoesNotMatch";

  public static final String NO_SUCH_BUCKET_POLICY_CODE = "NoSuchBucketPolicy";

  public static Map<String, String> ERROR_CODE_MESSAGES = ImmutableMap.of(
      INVALID_ACCESS_KEY_ID_CODE, "The Access Key ID provided was invalid.",
      ACCESS_DENIED_CODE, "Access denied. Please check the credentials provided have proper permissions.",
      SIGNATURE_DOES_NOT_MATCH_CODE, "The secret access key does not match causing an invalid signature.");

  private final String message;

  private S3BlobStoreException(
      final String message,
      final Throwable cause,
      @Nullable final BlobId blobId)
  {
    super(message, cause, blobId);
    this.message = message;
  }

  private S3BlobStoreException(final String message) {
    super(message, null);
    this.message = message;
  }

  public static S3BlobStoreException buildException(final S3Exception cause) {
    String message = ERROR_CODE_MESSAGES.getOrDefault(cause.awsErrorDetails().errorCode(), DEFAULT_MESSAGE);
    return new S3BlobStoreException(message, cause, null);
  }

  public static S3BlobStoreException insufficientCreatePermissionsError() {
    return new S3BlobStoreException(INSUFFICIENT_PERM_CREATE_BUCKET_ERR_MSG);
  }

  public static S3BlobStoreException unexpectedError(String action) {
    return new S3BlobStoreException(format(UNEXPECTED_ERR, action));
  }

  public static S3BlobStoreException bucketOwnershipError() {
    return new S3BlobStoreException(BUCKET_OWNERSHIP_ERR_MSG);
  }

  public static S3BlobStoreException invalidIdentityError() {
    return new S3BlobStoreException(INVALID_IDENTITY_ERR_MSG);
  }

  @Override
  public String getMessage() {
    return message;
  }
}
