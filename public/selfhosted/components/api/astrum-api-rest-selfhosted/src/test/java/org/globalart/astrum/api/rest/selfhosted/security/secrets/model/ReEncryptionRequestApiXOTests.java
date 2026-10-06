
package org.globalart.astrum.api.rest.selfhosted.security.secrets.model;

import java.util.Set;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;

import org.assertj.core.api.Assertions;
import org.junit.BeforeClass;
import org.junit.Test;

public class ReEncryptionRequestApiXOTests
{
  private static Validator validator;

  @BeforeClass
  public static void setup() {
    validator = Validation.buildDefaultValidatorFactory().getValidator();
  }

  @Test
  public void testValidationFailsEmptyKeyId() {
    ReEncryptionRequestApiXO xo = new ReEncryptionRequestApiXO();
    Set<ConstraintViolation<ReEncryptionRequestApiXO>> violations = validator.validate(xo);
    Assertions.assertThat(violations).hasSize(1);

    xo = new ReEncryptionRequestApiXO("", null);
    violations = validator.validate(xo);
    Assertions.assertThat(violations).hasSize(1);
  }

  @Test
  public void testValidationSucceed() {
    ReEncryptionRequestApiXO xo = new ReEncryptionRequestApiXO("test-key", null);
    Set<ConstraintViolation<ReEncryptionRequestApiXO>> violations = validator.validate(xo);
    Assertions.assertThat(violations).isEmpty();
  }

  @Test
  public void testFieldsSet() {
    ReEncryptionRequestApiXO xo = new ReEncryptionRequestApiXO("test-key", "mail@test.com");
    Set<ConstraintViolation<ReEncryptionRequestApiXO>> violations = validator.validate(xo);
    Assertions.assertThat(violations).isEmpty();
    Assertions.assertThat(xo.getSecretKeyId()).isEqualTo("test-key");
    Assertions.assertThat(xo.getNotifyEmail()).isEqualTo("mail@test.com");
  }
}
