
package org.globalart.astrum.validation.constraint;

import jakarta.validation.Validation;
import jakarta.validation.Validator;
import jakarta.validation.ConstraintViolation;

import org.junit.Before;
import org.junit.Test;

import java.util.Set;

import static org.globalart.astrum.validation.constraint.CaseType.LOWER;
import static org.globalart.astrum.validation.constraint.CaseType.UPPER;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;

public class CaseTest

{

  private Validator validator;

  @Before
  public void setUp() {
    validator = Validation.buildDefaultValidatorFactory().getValidator();
  }

  private static class UpperCaseSubject
  {
    @Case(UPPER)
    private String value;

    public UpperCaseSubject(String value) {
      this.value = value;
    }
  }

  @Test
  public void testUpperCase() {
    Set<ConstraintViolation<UpperCaseSubject>> violations;

    violations = validator.validate(new UpperCaseSubject("YES"));
    assertThat(violations, hasSize(0));

    violations = validator.validate(new UpperCaseSubject("No"));
    assertThat(violations, hasSize(1));
  }

  private static class LowerCaseSubject
  {
    @Case(LOWER)
    private String value;

    public LowerCaseSubject(String value) {
      this.value = value;
    }
  }

  @Test
  public void testLowerCase() {
    Set<ConstraintViolation<LowerCaseSubject>> violations;

    violations = validator.validate(new LowerCaseSubject("yes"));
    assertThat(violations, hasSize(0));

    violations = validator.validate(new LowerCaseSubject("No"));
    assertThat(violations, hasSize(1));
  }
}
