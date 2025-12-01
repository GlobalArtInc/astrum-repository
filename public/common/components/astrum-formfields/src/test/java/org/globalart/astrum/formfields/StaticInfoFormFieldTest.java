
package org.globalart.astrum.formfields;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

/**
 * {@link StaticInfoFormField} tests.
 */
public class StaticInfoFormFieldTest
{
  private static final String ID = "testId";

  private static final String LABEL = "testLabel";

  private static final String DESCRIPTION = "testDescription";

  private StaticInfoFormField formField;

  @Before
  public void setUp() {
    formField = new StaticInfoFormField(ID, LABEL, DESCRIPTION);
  }

  @Test
  public void when_CreatingNew_Expect_CorrectId() {
    assertThat(formField.getId(), equalTo(ID));
  }

  @Test
  public void when_CreatingNew_Expect_CorrectLabel() {
    assertThat(formField.getLabel(), equalTo(LABEL));
  }

  @Test
  public void when_CreatingNew_Expect_DescriptionStoredAsHelpText() {
    assertThat(formField.getHelpText(), equalTo(DESCRIPTION));
  }

  @Test
  public void when_CreatingNew_Expect_RequiredIsFalse() {
    assertFalse(formField.isRequired());
  }

  @Test
  public void when_CreatingNew_Expect_DisabledIsFalse() {
    assertFalse(formField.isDisabled());
  }

  @Test
  public void when_CreatingNew_Expect_ReadOnlyIsFalse() {
    assertFalse(formField.isReadOnly());
  }

  @Test
  public void when_CreatingNew_Expect_AllowAutocompleteIsFalse() {
    assertFalse(formField.getAllowAutocomplete());
  }

  @Test
  public void when_CreatingNew_Expect_OptionalFieldsAreNull() {
    assertThat(formField.getRegexValidation(), nullValue());
    assertThat(formField.getInitialValue(), nullValue());
  }

  @Test
  public void when_GettingAttributes_Expect_NonNullAndEmpty() {
    assertThat(formField.getAttributes(), is(notNullValue()));
    assertTrue(formField.getAttributes().isEmpty());
  }

  @Test
  public void when_GettingAttributes_Repeatedly_Expect_SameInstance() {
    assertThat(formField.getAttributes(), is(sameInstance(formField.getAttributes())));
  }

  @Test
  public void when_GettingType_Expect_StaticInfo() {
    assertThat(formField.getType(), equalTo("staticInfo"));
  }

  @Test
  public void when_CreatingNew_Expect_IsAbstractFormField() {
    assertThat(formField, instanceOf(AbstractFormField.class));
  }

  @Test
  public void when_CreatingNew_Expect_IsFormField() {
    assertThat(formField, instanceOf(FormField.class));
  }

  @Test
  public void when_CreatingNew_WithNullDescription_Expect_HelpTextIsNull() {
    StaticInfoFormField field = new StaticInfoFormField(ID, LABEL, null);
    assertThat(field.getHelpText(), nullValue());
    assertThat(field.getId(), equalTo(ID));
    assertThat(field.getLabel(), equalTo(LABEL));
    assertFalse(field.isRequired());
  }

  @Test
  public void when_CreatingNew_WithNullId_Expect_IdIsNull() {
    StaticInfoFormField field = new StaticInfoFormField(null, LABEL, DESCRIPTION);
    assertThat(field.getId(), nullValue());
    assertThat(field.getLabel(), equalTo(LABEL));
    assertThat(field.getHelpText(), equalTo(DESCRIPTION));
  }

  @Test
  public void when_CreatingNew_WithNullLabel_Expect_LabelIsNull() {
    StaticInfoFormField field = new StaticInfoFormField(ID, null, DESCRIPTION);
    assertThat(field.getLabel(), nullValue());
    assertThat(field.getId(), equalTo(ID));
    assertThat(field.getHelpText(), equalTo(DESCRIPTION));
  }
}
