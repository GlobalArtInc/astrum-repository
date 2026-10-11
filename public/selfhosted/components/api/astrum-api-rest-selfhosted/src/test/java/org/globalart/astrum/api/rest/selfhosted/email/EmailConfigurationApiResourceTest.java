
package org.globalart.astrum.api.rest.selfhosted.email;

import jakarta.validation.ValidationException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailConfiguration;
import org.globalart.astrum.api.rest.selfhosted.email.model.ApiEmailValidation;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.apache.commons.mail2.core.EmailException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class EmailConfigurationApiResourceTest
{
  @Mock
  private EmailManager emailManager;

  @Mock
  private AntiSsrfService antiSsrfService;

  @Mock
  private EmailConfiguration emailConfiguration;

  private EmailConfigurationApiResource underTest;

  @BeforeEach
  void setup() {
    underTest = new EmailConfigurationApiResource(emailManager, antiSsrfService);
  }

  @Test
  void getUnconfiguredEmailConfigurationHandlesNullDefaultConfiguration() {
    ApiEmailConfiguration response = underTest.getEmailConfiguration();

    assertThat(response.getFromAddress(), is(nullValue()));
    assertThat(response.getHost(), is(nullValue()));
    assertThat(response.getPassword(), is(nullValue()));
    assertThat(response.getPort(), is(nullValue()));
    assertThat(response.getSubjectPrefix(), is(nullValue()));
    assertThat(response.getUsername(), is(nullValue()));
    assertThat(response.isEnabled(), is(false));
    assertThat(response.isAstrumTrustStoreEnabled(), is(false));
    assertThat(response.isSslOnConnectEnabled(), is(false));
    assertThat(response.isSslServerIdentityCheckEnabled(), is(false));
    assertThat(response.isStartTlsEnabled(), is(false));
    assertThat(response.isStartTlsRequired(), is(false));
  }

  @Test
  void getEmailConfigurationObfuscatesThePassword() {
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);

    ApiEmailConfiguration response = underTest.getEmailConfiguration();

    assertThat(response.getPassword(), is(nullValue()));
  }

  @Test
  void setEmailConfigurationSetsTheNewConfiguration() {
    EmailConfiguration newConfiguration = mock(EmailConfiguration.class);
    String newPassword = "testPassword";
    ApiEmailConfiguration request = new ApiEmailConfiguration();
    request.setEnabled(true);
    request.setHost("smtp.example.com");
    request.setPassword(newPassword);

    when(emailManager.newConfiguration()).thenReturn(newConfiguration);

    underTest.setEmailConfiguration(request);

    verify(antiSsrfService).validateHostWithoutCache("smtp.example.com");
    verify(emailManager).setConfiguration(newConfiguration, newPassword);
    verify(newConfiguration).setEnabled(true);
  }

  @Test
  void setEmailConfigurationKeepsTheOriginalPassword() {
    EmailConfiguration newConfiguration = mock(EmailConfiguration.class);
    when(emailManager.newConfiguration()).thenReturn(newConfiguration);

    ApiEmailConfiguration request = new ApiEmailConfiguration();
    request.setEnabled(true);
    request.setPassword(Strings2.EMPTY);

    underTest.setEmailConfiguration(request);

    verify(newConfiguration).setEnabled(true);
    verify(emailManager).setConfiguration(newConfiguration, Strings2.EMPTY);
  }

  @Test
  void setEmailConfiguration_blocksPrivateSmtpHost() {
    ApiEmailConfiguration request = new ApiEmailConfiguration();
    request.setHost("192.168.1.1");
    doThrow(new ValidationException("SSRF protection: private network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("192.168.1.1");

    ValidationErrorsException error =
        assertThrows(ValidationErrorsException.class, () -> underTest.setEmailConfiguration(request));

    // answered as a 400 naming the host, not as a server error
    assertThat(error.getValidationErrors().get(0).getId(), is("host"));
    assertThat(error.getValidationErrors().get(0).getMessage(), is("SSRF protection: private network address"));
    verify(emailManager, never()).setConfiguration(any(), any());
  }

  @Test
  void testEmailConfiguration_Success() throws EmailException {
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);
    String destinationAddress = "test@example.com";

    Response response = underTest.testEmailConfiguration(destinationAddress);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(200));
    assertThat(response.hasEntity(), is(true));

    ApiEmailValidation validation = (ApiEmailValidation) response.getEntity();
    assertThat(validation.isSuccess(), is(true));
    assertThat(validation.getReason(), is(""));

    verify(emailManager).sendVerification(emailConfiguration, destinationAddress);
  }

  @Test
  void testEmailConfiguration_NullConfiguration() {
    when(emailManager.getConfiguration()).thenReturn(null);
    String destinationAddress = "test@example.com";

    Response response = underTest.testEmailConfiguration(destinationAddress);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(400));
    assertThat(response.hasEntity(), is(true));

    ApiEmailValidation validation = (ApiEmailValidation) response.getEntity();
    assertThat(validation.isSuccess(), is(false));
    assertThat(validation.getReason(), is("Email Settings are not yet configured"));
  }

  @Test
  void testEmailConfiguration_EmailExceptionWithRootCause() throws EmailException {
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);
    String destinationAddress = "test@example.com";

    IllegalStateException rootCause = new IllegalStateException("Connection refused");
    EmailException emailException = new EmailException("Failed to send email", rootCause);
    doThrow(emailException).when(emailManager).sendVerification(emailConfiguration, destinationAddress);

    Response response = underTest.testEmailConfiguration(destinationAddress);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(400));
    assertThat(response.hasEntity(), is(true));

    ApiEmailValidation validation = (ApiEmailValidation) response.getEntity();
    assertThat(validation.isSuccess(), is(false));
    assertThat(validation.getReason(), is("Connection refused"));
  }

  @Test
  void testEmailConfiguration_EmailExceptionWithoutRootCause() throws EmailException {
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);
    String destinationAddress = "test@example.com";

    EmailException emailException = new EmailException("Invalid email address");
    doThrow(emailException).when(emailManager).sendVerification(emailConfiguration, destinationAddress);

    Response response = underTest.testEmailConfiguration(destinationAddress);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(400));
    assertThat(response.hasEntity(), is(true));

    ApiEmailValidation validation = (ApiEmailValidation) response.getEntity();
    assertThat(validation.isSuccess(), is(false));
    assertThat(validation.getReason(), is("Invalid email address"));
  }

  @Test
  void testEmailConfiguration_EmailExceptionWithNullMessage() throws EmailException {
    when(emailManager.getConfiguration()).thenReturn(emailConfiguration);
    String destinationAddress = "test@example.com";

    IllegalStateException causeWithNoMessage = new IllegalStateException();
    EmailException emailException = new EmailException("wrapper", causeWithNoMessage);
    doThrow(emailException).when(emailManager).sendVerification(emailConfiguration, destinationAddress);

    Response response = underTest.testEmailConfiguration(destinationAddress);

    assertThat(response, is(notNullValue()));
    assertThat(response.getStatus(), is(400));
    assertThat(response.hasEntity(), is(true));

    ApiEmailValidation validation = (ApiEmailValidation) response.getEntity();
    assertThat(validation.isSuccess(), is(false));
    assertThat(validation.getReason(), is("IllegalStateException"));
  }
}
