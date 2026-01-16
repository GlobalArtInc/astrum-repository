
package org.globalart.astrum.siesta.internal;

import java.util.Collections;
import java.util.List;

import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;

class ValidationErrorsExceptionMapperTest
{
  private ValidationErrorsExceptionMapper mapper;

  @BeforeEach
  void setup() {
    mapper = new ValidationErrorsExceptionMapper();
  }

  @Test
  void testGetValidationErrors() {
    ValidationErrorXO error = new ValidationErrorXO("testField", "test error");
    ValidationErrorsException exception = new ValidationErrorsException();
    exception.withErrors(Collections.singletonList(error));

    List<ValidationErrorXO> errors = mapper.getValidationErrors(exception);

    assertThat(errors, hasSize(1));
    assertThat(errors.get(0).getId(), equalTo("testField"));
    assertThat(errors.get(0).getMessage(), equalTo("test error"));
  }
}
