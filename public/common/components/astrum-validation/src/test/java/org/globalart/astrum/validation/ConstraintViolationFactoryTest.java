
package org.globalart.astrum.validation;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.ValidatorFactory;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.bootstrap.validation.ValidationConfiguration;
import org.globalart.astrum.validation.internal.SpringConstraintValidatorFactory;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.InjectMocks;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.springframework.beans.factory.NoSuchBeanDefinitionException;
import org.springframework.context.ApplicationContext;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ConstraintViolationFactoryTest

{
  private static final String JAVA_EL_IMMEDIATE = "${2 + 2}";

  private static final String JAVA_EL_DEFERRED = "{message}";

  private static final String ANY_PATH = "foo";

  private ValidationConfiguration configuration = new ValidationConfiguration();

  @Mock
  private ApplicationContext context;

  @InjectMocks
  private SpringConstraintValidatorFactory constraintValidatorFactory;

  private ConstraintViolationFactory cvf;

  @Before
  public void setUp() throws Exception {
    when(context.getBean(any(Class.class))).thenThrow(new NoSuchBeanDefinitionException(NotNull.class));
    ValidatorFactory factory = configuration.validatorFactory(constraintValidatorFactory);
    cvf = new ConstraintViolationFactory(() -> configuration.validator(factory));
  }

  @After
  public void teardown() {
    ValidationConfiguration.EXECUTABLE_VALIDATOR = null;
  }

  @Test
  public void shouldStripJavaExpression() {
    // immediate evaluation of JAVA_EL_IMMEDIATE would yield 4, and that would mean we are open to code injection
    ConstraintViolation<?> violation = cvf.createViolation(ANY_PATH, JAVA_EL_IMMEDIATE);
    assertThat(violation.getMessage(), is("{2 + 2}"));
  }

  @Test
  public void shouldEvaluateDeferredExpressionLanguage() {
    // deferred evaluation of JAVA_EL_DEFERRED should yield content of HelperAnnotation.message
    ConstraintViolation<?> violation = cvf.createViolation(ANY_PATH, JAVA_EL_DEFERRED);
    assertThat(violation.getMessage(), is(""));
  }
}
