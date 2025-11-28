
package org.globalart.astrum.formfields;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertFalse;
import static org.hamcrest.Matchers.equalTo;

/**
 * {@link AbstractFormField} tests.
 */
public class AbstractFormFieldTest
{
  private static final String ID = "testId";

  private static final String TYPE = "testField";

  private AbstractFormField<String> formField;

  @Before
  public void setUp() {
    formField = new AbstractFormField<String>(ID)
    {
      @Override
      public String getType() {
        return TYPE;
      }
    };
  }

  @Test
  public void when_CreatingNew_WithId_Expect_CorrectId() {
    assertThat(formField.getId(), equalTo(ID));
  }

  @Test
  public void when_CreatingNew_WithId_Expect_CorrectType() {
    assertThat(formField.getType(), equalTo(TYPE));
  }

  @Test
  public void when_CreatingNew_WithId_Expect_RequiredIsFalse() {
    assertFalse(formField.isRequired());
  }

  @Test
  public void when_CreatingNew_WithId_Expect_DisabledIsFalse() {
    assertFalse(formField.isDisabled());
  }

  @Test
  public void when_CreatingNew_WithId_Expect_IsReadOnlyIsFalse() {
    assertFalse(formField.isReadOnly());
  }
}
