
package org.globalart.astrum.validation.constraint;

import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ConstraintViolation;

import org.junit.Before;
import org.junit.Test;

import java.util.Set;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;

public class PortNumberTest

{

  private Validator validator;

  @Before
  public void setUp() {
    validator = Validation.buildDefaultValidatorFactory().getValidator();
  }

  private static class MaxPortSubject
  {
    @PortNumber(max = 100)
    private Integer value;

    public MaxPortSubject(Integer value) {
      this.value = value;
    }
  }

  @Test
  public void testMaxPort() {
    Set<ConstraintViolation<MaxPortSubject>> violations;

    violations = validator.validate(new MaxPortSubject(100));
    assertThat(violations, hasSize(0));

    violations = validator.validate(new MaxPortSubject(101));
    assertThat(violations, hasSize(1));
  }
}
