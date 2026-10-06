
package org.globalart.astrum.api.rest.selfhosted.blobstore.file;

import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validator;

import org.globalart.astrum.api.rest.common.blobstore.file.model.FileBlobStoreApiCreateRequest;
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
 * Tests for validating FileBlobStoreResource endpoint validation.
 */
@ExtendWith(ValidationExtension.class)
class FileBlobStoreResourceValidationTest
{
  @ValidationExecutor
  private final Validator validator =
      new ValidationConfiguration().validatorFactory(new ConstraintValidatorFactoryImpl()).getValidator();

  @Test
  void testCreateRequestRejectsNamesStartingWithUnderscore() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName("_invalidName");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testCreateRequestRejectsNamesStartingWithDot() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName(".invalidName");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testCreateRequestRejectsNamesWithSpecialCharacters() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName("invalid@name");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat(violations.size(), is(greaterThanOrEqualTo(1)));
    boolean hasNameViolation = violations.stream()
        .anyMatch(v -> "name".equals(v.getPropertyPath().toString()));
    assertThat("Should have a name validation violation", hasNameViolation, is(true));
  }

  @Test
  void testCreateRequestAcceptsValidNames() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName("valid-name");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat("Valid name should not have violations", violations.isEmpty(), is(true));
  }

  @Test
  void testCreateRequestAcceptsNamesWithUnderscoresInMiddle() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName("valid_name");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat("Name with underscores in middle should be valid", violations.isEmpty(), is(true));
  }

  @Test
  void testCreateRequestAcceptsNamesWithDotsInMiddle() {
    FileBlobStoreApiCreateRequest request = new FileBlobStoreApiCreateRequest();
    request.setName("valid.name");
    request.setPath("/tmp/blobstore");

    Set<ConstraintViolation<FileBlobStoreApiCreateRequest>> violations = validator.validate(request);

    assertThat("Name with dots in middle should be valid", violations.isEmpty(), is(true));
  }
}
