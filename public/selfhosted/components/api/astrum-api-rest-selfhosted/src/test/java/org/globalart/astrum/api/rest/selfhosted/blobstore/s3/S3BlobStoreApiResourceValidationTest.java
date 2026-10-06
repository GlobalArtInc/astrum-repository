
package org.globalart.astrum.api.rest.selfhosted.blobstore.s3;

import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validator;

import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucket;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiBucketConfiguration;
import org.globalart.astrum.api.rest.common.blobstore.s3.model.S3BlobStoreApiModel;
import org.globalart.astrum.bootstrap.validation.ValidationConfiguration;
import org.globalart.astrum.testcommon.validation.ValidationExtension;
import org.globalart.astrum.testcommon.validation.ValidationExtension.ValidationExecutor;

import org.hibernate.validator.internal.engine.constraintvalidation.ConstraintValidatorFactoryImpl;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.greaterThanOrEqualTo;
import static org.hamcrest.Matchers.is;

/**
 * Tests for validating S3BlobStoreApiResource endpoint validation.
 */
@ExtendWith(ValidationExtension.class)
class S3BlobStoreApiResourceValidationTest
{
  @ValidationExecutor
  private final Validator validator =
      new ValidationConfiguration().validatorFactory(new ConstraintValidatorFactoryImpl()).getValidator();

  @Test
  void testS3BlobStoreRejectsNamesStartingWithUnderscore() {
    S3BlobStoreApiBucket bucket = new S3BlobStoreApiBucket("us-east-1", "my-bucket", "prefix");
    S3BlobStoreApiBucketConfiguration config =
        new S3BlobStoreApiBucketConfiguration(bucket, null, null, null, null, null, null);
    S3BlobStoreApiModel model = new S3BlobStoreApiModel("_invalidS3Name", null, config);

    Set<ConstraintViolation<S3BlobStoreApiModel>> violations = validator.validate(model);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testS3BlobStoreRejectsNamesStartingWithDot() {
    S3BlobStoreApiBucket bucket = new S3BlobStoreApiBucket("us-east-1", "my-bucket", "prefix");
    S3BlobStoreApiBucketConfiguration config =
        new S3BlobStoreApiBucketConfiguration(bucket, null, null, null, null, null, null);
    S3BlobStoreApiModel model = new S3BlobStoreApiModel(".invalidS3Name", null, config);

    Set<ConstraintViolation<S3BlobStoreApiModel>> violations = validator.validate(model);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testS3BlobStoreRejectsNamesWithSlashes() {
    S3BlobStoreApiBucket bucket = new S3BlobStoreApiBucket("us-east-1", "my-bucket", "prefix");
    S3BlobStoreApiBucketConfiguration config =
        new S3BlobStoreApiBucketConfiguration(bucket, null, null, null, null, null, null);
    S3BlobStoreApiModel model = new S3BlobStoreApiModel("invalid/s3/name", null, config);

    Set<ConstraintViolation<S3BlobStoreApiModel>> violations = validator.validate(model);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testS3BlobStoreAcceptsValidNames() {
    S3BlobStoreApiBucket bucket = new S3BlobStoreApiBucket("us-east-1", "my-bucket", "prefix");
    S3BlobStoreApiBucketConfiguration config =
        new S3BlobStoreApiBucketConfiguration(bucket, null, null, null, null, null, null);
    S3BlobStoreApiModel model = new S3BlobStoreApiModel("valid-s3-name", null, config);

    Set<ConstraintViolation<S3BlobStoreApiModel>> violations = validator.validate(model);

    assertThat("Valid S3 name should not have violations", violations.isEmpty(), is(true));
  }

  @Test
  void testS3BlobStoreAcceptsNamesWithUnderscoresAndDots() {
    S3BlobStoreApiBucket bucket = new S3BlobStoreApiBucket("us-east-1", "my-bucket", "prefix");
    S3BlobStoreApiBucketConfiguration config =
        new S3BlobStoreApiBucketConfiguration(bucket, null, null, null, null, null, null);
    S3BlobStoreApiModel model = new S3BlobStoreApiModel("valid_s3.name", null, config);

    Set<ConstraintViolation<S3BlobStoreApiModel>> violations = validator.validate(model);

    assertThat("S3 name with underscores and dots should be valid", violations.isEmpty(), is(true));
  }
}
