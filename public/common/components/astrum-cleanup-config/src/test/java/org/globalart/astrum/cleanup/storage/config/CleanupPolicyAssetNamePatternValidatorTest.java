
package org.globalart.astrum.cleanup.storage.config;

import jakarta.validation.ConstraintValidatorContext;
import jakarta.validation.ConstraintValidatorContext.ConstraintViolationBuilder;

import org.globalart.astrum.cleanup.storage.config.RegexCriteriaValidator.InvalidExpressionException;
import org.globalart.astrum.common.template.EscapeHelper;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InOrder;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CleanupPolicyAssetNamePatternValidatorTest
{
  private static final String VALID_PATTERN = "org/globalart";

  private static final String INVALID_PATTERN = "hello(world";

  private static final String INVALID_PATTERN_WITH_JAVA_EL = "hello(world${0}";

  private CleanupPolicyAssetNamePatternValidator underTest;

  @Mock
  private ConstraintValidatorContext context;

  @Mock
  private ConstraintViolationBuilder builder;

  @Before
  public void setUp() {
    underTest = new CleanupPolicyAssetNamePatternValidator();
  }

  @Test
  public void nullValueIsValidWithoutValidation() {
    assertThat(underTest.isValid(null, context), is(true));

    verifyNoInteractions(context);
  }

  @Test
  public void emptyValueIsValidWithoutBuildingViolation() {
    // empty string is a valid regular expression, so validation runs but passes
    assertThat(underTest.isValid("", context), is(true));

    verifyNoInteractions(context);
  }

  @Test
  public void whitespaceValueIsValidWithoutBuildingViolation() {
    // whitespace is a valid regular expression, so validation runs but passes
    assertThat(underTest.isValid("   ", context), is(true));

    verifyNoInteractions(context);
  }

  @Test
  public void validPatternIsValid() {
    assertThat(underTest.isValid(VALID_PATTERN, context), is(true));

    verifyNoInteractions(context);
  }

  @Test
  public void invalidPatternIsInvalidAndBuildsConstraintViolation() {
    when(context.buildConstraintViolationWithTemplate(any())).thenReturn(builder);

    String expectedTemplate = new EscapeHelper().stripJavaEl(invalidExpressionMessage(INVALID_PATTERN));

    boolean isValid = underTest.isValid(INVALID_PATTERN, context);

    assertThat(isValid, is(false));

    // the violation must be built from exactly these calls, invoked in this order
    ArgumentCaptor<String> templateCaptor = ArgumentCaptor.forClass(String.class);

    InOrder inOrder = inOrder(context, builder);
    inOrder.verify(context).disableDefaultConstraintViolation();
    inOrder.verify(context).buildConstraintViolationWithTemplate(templateCaptor.capture());
    inOrder.verify(builder).addConstraintViolation();

    // the template is the (real) escape-helper stripped exception message
    assertThat(templateCaptor.getValue(), is(expectedTemplate));
  }

  @Test
  public void invalidPatternWithJavaElIsStrippedFromViolationMessage() {
    when(context.buildConstraintViolationWithTemplate(any())).thenReturn(builder);

    String rawMessage = invalidExpressionMessage(INVALID_PATTERN_WITH_JAVA_EL);
    assertThat("precondition: raw message contains a Java EL start token", rawMessage.contains("${"), is(true));

    boolean isValid = underTest.isValid(INVALID_PATTERN_WITH_JAVA_EL, context);

    assertThat(isValid, is(false));

    ArgumentCaptor<String> templateCaptor = ArgumentCaptor.forClass(String.class);
    verify(context).buildConstraintViolationWithTemplate(templateCaptor.capture());

    // getEscapeHelper().stripJavaEl(...) actually runs against the real escape helper field
    assertThat(templateCaptor.getValue(), is(new EscapeHelper().stripJavaEl(rawMessage)));
    assertThat(templateCaptor.getValue().contains("${"), is(false));
  }

  /**
   * Returns the {@link InvalidExpressionException} message produced by the real validation code path for the given
   * pattern, so expectations are pinned to actual behavior rather than a hard-coded (JDK-specific) message.
   */
  private static String invalidExpressionMessage(final String pattern) {
    try {
      RegexCriteriaValidator.validate(pattern);
      throw new AssertionError("Expected an InvalidExpressionException for pattern: " + pattern);
    }
    catch (InvalidExpressionException e) {
      return e.getMessage();
    }
  }
}
