
package org.globalart.astrum.capability;

import java.net.URL;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.common.template.TemplateParameters;
import org.globalart.astrum.common.template.TemplateThrowableAdapter;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.PasswordFormField;
import org.globalart.astrum.formfields.StringTextFormField;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertNull;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CapabilitySupportTest
{
  @Mock
  private CapabilityContext context;

  @Mock
  private CapabilityDescriptor descriptor;

  @Mock
  private TemplateHelper templateHelper;

  @Mock
  private SecretsService secretsService;

  private TestCapability underTest;

  @Before
  public void setup() {
    underTest = new TestCapability();
    underTest.init(context, secretsService);
    underTest.setTemplateHelper(templateHelper);
  }

  @Test
  public void renderFailureReturnsNullOnFailure() {
    TemplateParameters params = new TemplateParameters()
        .set("cause", new TemplateThrowableAdapter(new Exception()));
    assertNull(underTest.render("missing-template.vm", params));
    verifyNoInteractions(templateHelper);
  }

  @Test
  public void renderFailureSuccess() {
    when(templateHelper.render(any(URL.class), any(TemplateParameters.class))).thenReturn("rendered");
    TemplateParameters params = new TemplateParameters()
        .set("cause", new TemplateThrowableAdapter(new Exception()));
    assertEquals("rendered", underTest.render("failure.vm", params));
    verify(templateHelper).render(underTest.getClass().getResource("failure.vm"), params);
  }

  @Test
  public void isPasswordPropertyReturnsTrueForEncryptedField() {
    PasswordFormField secretField = new PasswordFormField("secret", "Secret", "help", false);
    when(context.descriptor()).thenReturn(descriptor);
    when(descriptor.formFields()).thenReturn(List.<FormField>of(secretField));

    assertTrue(underTest.isPasswordProperty("secret"));
  }

  @Test
  public void isPasswordPropertyReturnsFalseForNonEncryptedField() {
    StringTextFormField nameField = new StringTextFormField("name", "Name", "help", false);
    when(context.descriptor()).thenReturn(descriptor);
    when(descriptor.formFields()).thenReturn(List.<FormField>of(nameField));

    assertFalse(underTest.isPasswordProperty("name"));
  }

  @Test
  public void isPasswordPropertyReturnsFalseForUnknownProperty() {
    when(context.descriptor()).thenReturn(descriptor);
    when(descriptor.formFields()).thenReturn(List.<FormField>of());

    assertFalse(underTest.isPasswordProperty("unknown"));
  }

  @Test
  public void isPasswordPropertyReturnsFalseWhenFormFieldsIsNull() {
    when(context.descriptor()).thenReturn(descriptor);
    when(descriptor.formFields()).thenReturn(null);

    assertFalse(underTest.isPasswordProperty("secret"));
  }

  private class TestCapability
      extends CapabilitySupport<TestCapabilityConfig>
  {
    @Override
    protected TestCapabilityConfig createConfig(final Map<String, String> properties) throws Exception {
      return new TestCapabilityConfig();
    }
  }

  private class TestCapabilityConfig
      extends CapabilityConfigurationSupport
  {
  }
}
