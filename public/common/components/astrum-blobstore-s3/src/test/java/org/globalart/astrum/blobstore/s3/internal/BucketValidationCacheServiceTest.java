
package org.globalart.astrum.blobstore.s3.internal;

import java.util.concurrent.ExecutionException;

import org.globalart.astrum.blobstore.s3.internal.BucketValidationCacheService.BucketValidationResult;

import com.google.common.util.concurrent.UncheckedExecutionException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import software.amazon.awssdk.awscore.exception.AwsErrorDetails;
import software.amazon.awssdk.services.s3.model.GetBucketPolicyResponse;
import software.amazon.awssdk.services.s3.model.S3Exception;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStoreException.ACCESS_DENIED_CODE;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStoreException.METHOD_NOT_ALLOWED_CODE;
import static org.globalart.astrum.blobstore.s3.internal.S3BlobStoreException.NO_SUCH_BUCKET_POLICY_CODE;

/**
 * {@link BucketValidationCacheService} tests.
 */
@ExtendWith(MockitoExtension.class)
class BucketValidationCacheServiceTest
{
  @Mock
  private EncryptingS3Client s3;

  @Mock
  private BucketOwnershipCheckFeatureFlag featureFlag;

  private BucketValidationCacheService underTest;

  @BeforeEach
  void setup() {
    lenient().when(featureFlag.isDisabled()).thenReturn(false);
    underTest = new BucketValidationCacheService(5, 100, featureFlag);
  }

  @Test
  void testBucketExistsAndOwnershipValid() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    when(s3.getBucketPolicy("my-bucket")).thenReturn(mock(GetBucketPolicyResponse.class));

    BucketValidationResult result = underTest.validate(s3, null, "my-bucket");

    assertThat(result.exists(), is(true));
    assertThat(result.ownershipValid(), is(true));
  }

  @Test
  void testBucketDoesNotExist() throws ExecutionException {
    when(s3.doesBucketExist("non-existent-bucket")).thenReturn(false);

    BucketValidationResult result = underTest.validate(s3, null, "non-existent-bucket");

    assertThat(result.exists(), is(false));
    assertThat(result.ownershipValid(), is(false));
    verify(s3, never()).getBucketPolicy(anyString());
  }

  @Test
  void testBucketExistsWithNoSuchBucketPolicyCode() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    S3Exception exception = mock(S3Exception.class);
    AwsErrorDetails details = AwsErrorDetails.builder()
        .errorCode(NO_SUCH_BUCKET_POLICY_CODE)
        .build();
    when(exception.awsErrorDetails()).thenReturn(details);
    when(s3.getBucketPolicy("my-bucket")).thenThrow(exception);

    BucketValidationResult result = underTest.validate(s3, null, "my-bucket");

    assertThat(result.exists(), is(true));
    assertThat(result.ownershipValid(), is(true));
  }

  @Test
  void testBucketExistsWithAccessDeniedCode() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    S3Exception exception = mock(S3Exception.class);
    AwsErrorDetails details = AwsErrorDetails.builder()
        .errorCode(ACCESS_DENIED_CODE)
        .build();
    when(exception.awsErrorDetails()).thenReturn(details);
    when(s3.getBucketPolicy("my-bucket")).thenThrow(exception);

    BucketValidationResult result = underTest.validate(s3, null, "my-bucket");

    assertThat(result.exists(), is(true));
    assertThat(result.ownershipValid(), is(false));
  }

  @Test
  void testBucketExistsWithMethodNotAllowedCode() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    S3Exception exception = mock(S3Exception.class);
    AwsErrorDetails details = AwsErrorDetails.builder()
        .errorCode(METHOD_NOT_ALLOWED_CODE)
        .build();
    when(exception.awsErrorDetails()).thenReturn(details);
    when(s3.getBucketPolicy("my-bucket")).thenThrow(exception);

    BucketValidationResult result = underTest.validate(s3, null, "my-bucket");

    assertThat(result.exists(), is(true));
    assertThat(result.ownershipValid(), is(false));
  }

  @Test
  void testBucketExistsWithUnexpectedError() {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    S3Exception exception = mock(S3Exception.class);
    AwsErrorDetails details = AwsErrorDetails.builder()
        .errorCode("UnexpectedErrorCode")
        .build();
    when(exception.awsErrorDetails()).thenReturn(details);
    when(s3.getBucketPolicy("my-bucket")).thenThrow(exception);

    UncheckedExecutionException thrown = assertThrows(UncheckedExecutionException.class,
        () -> underTest.validate(s3, null, "my-bucket"));
    assertThat(thrown.getCause(), is(instanceOf(S3Exception.class)));
  }

  @Test
  void testOwnershipCheckDisabledByFeatureFlag() throws ExecutionException {
    when(featureFlag.isDisabled()).thenReturn(true);
    underTest = new BucketValidationCacheService(5, 100, featureFlag);

    when(s3.doesBucketExist("my-bucket")).thenReturn(true);

    BucketValidationResult result = underTest.validate(s3, null, "my-bucket");

    assertThat(result.exists(), is(true));
    assertThat(result.ownershipValid(), is(true));
    verify(s3, never()).getBucketPolicy(anyString());
  }

  @Test
  void testCacheUsedOnSecondCall() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    when(s3.getBucketPolicy("my-bucket")).thenReturn(mock(GetBucketPolicyResponse.class));

    underTest.validate(s3, null, "my-bucket");
    underTest.validate(s3, null, "my-bucket");

    verify(s3, times(1)).doesBucketExist("my-bucket");
    verify(s3, times(1)).getBucketPolicy("my-bucket");
  }

  @Test
  void testInvalidateMethod() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    when(s3.getBucketPolicy("my-bucket")).thenReturn(mock(GetBucketPolicyResponse.class));

    underTest.validate(s3, null, "my-bucket");
    underTest.invalidate(null, "my-bucket");
    underTest.validate(s3, null, "my-bucket");

    verify(s3, times(2)).doesBucketExist("my-bucket");
    verify(s3, times(2)).getBucketPolicy("my-bucket");
  }

  @Test
  void testDifferentEndpoints() throws ExecutionException {
    when(s3.doesBucketExist("my-bucket")).thenReturn(true);
    when(s3.getBucketPolicy("my-bucket")).thenReturn(mock(GetBucketPolicyResponse.class));
    EncryptingS3Client other = mock();
    when(other.doesBucketExist("my-bucket")).thenReturn(true);
    when(other.getBucketPolicy("my-bucket")).thenReturn(mock(GetBucketPolicyResponse.class));

    underTest.validate(s3, "http://foo.minio.dev", "my-bucket");
    underTest.validate(other, "http://bar.minio.dev", "my-bucket");

    verify(s3).doesBucketExist("my-bucket");
    verify(s3).getBucketPolicy("my-bucket");

    verify(other).doesBucketExist("my-bucket");
    verify(other).getBucketPolicy("my-bucket");
  }
}
