
package org.globalart.astrum.validation;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.ValidatorFactory;
import jakarta.validation.constraints.NotNull;

import org.junit.Before;
import org.junit.Test;
import org.globalart.astrum.validation.internal.AopAwareParanamerParameterNameProvider;

import java.lang.reflect.Method;
import java.util.Set;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;

public class ParanamerValidationTest

{

  private ValidatorFactory factory;

  @Before
  public void setUp() {
    factory = Validation.byDefaultProvider()
        .configure()
        .parameterNameProvider(new AopAwareParanamerParameterNameProvider())
        .buildValidatorFactory();
  }

  private static class TestSubject
  {
    public void test(@NotNull String text) {
      // ignore
    }
  }

  @Test
  public void validateMethodResolveParameterName() throws NoSuchMethodException {
    Method method = TestSubject.class.getMethod("test", String.class);
    TestSubject obj = new TestSubject();

    Object[] values = {null};
    Class<?>[] groups = {};
    Set<ConstraintViolation<TestSubject>> violations =
        factory.getValidator().forExecutables().validateParameters(obj, method, values, groups);
    assertThat(violations, hasSize(1));
    ConstraintViolation<TestSubject> violation = violations.iterator().next();

    // should be 'text' instead of 'arg0'
    assertThat(violation.getPropertyPath().toString(), equalTo("test.text"));
  }
}
