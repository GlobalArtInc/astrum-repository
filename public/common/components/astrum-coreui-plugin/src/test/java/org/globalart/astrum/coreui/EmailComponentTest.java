
package org.globalart.astrum.coreui;

import jakarta.validation.ValidationException;
import jakarta.validation.Validator;

import org.globalart.astrum.bootstrap.validation.ValidationConfiguration;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.email.EmailManager;
import org.globalart.astrum.rapture.PasswordPlaceholder;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;
import org.globalart.astrum.testcommon.validation.ValidationExtension.ValidationExecutor;
import org.globalart.astrum.validation.ssrf.AntiSsrfService;

import org.hibernate.validator.internal.engine.constraintvalidation.ConstraintValidatorFactoryImpl;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(ValidationExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class EmailComponentTest
{
  private static final String ADDRESS = "you@somewhere.com";

  @ValidationExecutor
  private final Validator validator =
      new ValidationConfiguration().validatorFactory(new ConstraintValidatorFactoryImpl()).getValidator();

  @Mock
  private EmailManager emailManager;

  @Mock
  private AntiSsrfService antiSsrfService;

  @Mock
  private EmailConfiguration emailConfiguration;

  @InjectMocks
  private EmailComponent underTest;

  @BeforeEach
  void setUp() {
    lenient().when(emailManager.getConfiguration()).thenReturn(emailConfiguration);
    lenient().when(emailConfiguration.isEnabled()).thenReturn(false);
    lenient().when(emailConfiguration.getHost()).thenReturn("localhost");
    lenient().when(emailConfiguration.getPort()).thenReturn(25);
    lenient().when(emailConfiguration.getFromAddress()).thenReturn("astrum@example.org");
    lenient().when(emailConfiguration.getUsername()).thenReturn("foo");
    lenient().when(emailConfiguration.isStartTlsEnabled()).thenReturn(true);
    lenient().when(emailConfiguration.isStartTlsRequired()).thenReturn(true);
    lenient().when(emailConfiguration.isSslOnConnectEnabled()).thenReturn(true);
    lenient().when(emailConfiguration.getSubjectPrefix()).thenReturn("prefix");
  }

  @Test
  void readReturnsCurrentConfigurationWithPasswordPlaceHolder() {
    when(emailConfiguration.getPassword()).thenReturn(mock(Secret.class));

    EmailConfigurationXO actualConfig = underTest.read();

    assertThat(actualConfig, is(notNullValue()));
    assertThat(actualConfig.isEnabled(), is(false));
    assertThat(actualConfig.getHost(), is("localhost"));
    assertThat(actualConfig.getPort(), is(25));
    assertThat(actualConfig.getUsername(), is("foo"));
    assertThat(actualConfig.getFromAddress(), is("astrum@example.org"));
    assertThat(actualConfig.getPassword(), is(PasswordPlaceholder.get()));
    assertThat(actualConfig.isStartTlsEnabled(), is(true));
    assertThat(actualConfig.isStartTlsRequired(), is(true));
    assertThat(actualConfig.isSslOnConnectEnabled(), is(true));
    assertThat(actualConfig.getSubjectPrefix(), is("prefix"));
    assertThat(actualConfig.isAstrumTrustStoreEnabled(), is(false));
    assertThat(actualConfig.isSslCheckServerIdentityEnabled(), is(false));
  }

  @Test
  void readReturnsCurrentConfigurationWithEmptyPasswordPlaceHolder() {
    when(emailConfiguration.getPassword()).thenReturn(null);

    EmailConfigurationXO actualConfig = underTest.read();

    assertThat(actualConfig, is(notNullValue()));
    assertThat(actualConfig.isEnabled(), is(false));
    assertThat(actualConfig.getHost(), is("localhost"));
    assertThat(actualConfig.getPort(), is(25));
    assertThat(actualConfig.getUsername(), is("foo"));
    assertThat(actualConfig.getFromAddress(), is("astrum@example.org"));
    assertThat(actualConfig.getPassword(), is(nullValue()));
    assertThat(actualConfig.isStartTlsEnabled(), is(true));
    assertThat(actualConfig.isStartTlsRequired(), is(true));
    assertThat(actualConfig.isSslOnConnectEnabled(), is(true));
    assertThat(actualConfig.getSubjectPrefix(), is("prefix"));
    assertThat(actualConfig.isAstrumTrustStoreEnabled(), is(false));
    assertThat(actualConfig.isSslCheckServerIdentityEnabled(), is(false));
  }

  @Test
  void updateSavesConfiguration() {
    when(emailManager.newConfiguration()).thenReturn(emailConfiguration);

    underTest.update(getConfigurationXO("baz"));

    verify(antiSsrfService).validateHostWithoutCache("localhost");
    verify(emailManager).setConfiguration(emailConfiguration, "baz");
    verify(emailManager).getConfiguration();
  }

  @Test
  void update_blocksPrivateSmtpHost() {
    doThrow(new ValidationException("SSRF protection: private network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("localhost");

    assertThrows(ValidationException.class, () -> underTest.update(getConfigurationXO("baz")));

    verify(emailManager, never()).setConfiguration(any(), any());
  }

  @Test
  void sendVerification() throws Exception {
    EmailConfigurationXO formCredentials = getConfigurationXO("baz");
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailManager.newConfiguration()).thenReturn(emailConfig);

    underTest.sendVerification(formCredentials, ADDRESS);

    verify(antiSsrfService).validateHostWithoutCache("localhost");
    verify(emailManager).sendVerification(emailConfig, "baz", ADDRESS);
  }

  @Test
  void sendVerification_blocksLoopbackHost() throws Exception {
    EmailConfigurationXO configuration = getConfigurationXO("baz");
    doThrow(new ValidationException("SSRF protection: private network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("localhost");

    assertThrows(ValidationException.class, () -> underTest.sendVerification(configuration, ADDRESS));

    verify(emailManager, never()).sendVerification(any(), any(), any());
  }

  @Test
  void sendVerification_blocksPrivateIpHost() throws Exception {
    EmailConfigurationXO configuration = getConfigurationXOWithHost("192.168.1.1", "baz");
    doThrow(new ValidationException("SSRF protection: private network address"))
        .when(antiSsrfService)
        .validateHostWithoutCache("192.168.1.1");

    assertThrows(ValidationException.class, () -> underTest.sendVerification(configuration, ADDRESS));

    verify(emailManager, never()).sendVerification(any(), any(), any());
  }

  @Test
  void sendVerification_allowsPublicSmtpHost() throws Exception {
    EmailConfigurationXO configuration = getConfigurationXOWithHost("smtp.example.com", "baz");
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailManager.newConfiguration()).thenReturn(emailConfig);

    underTest.sendVerification(configuration, ADDRESS);

    verify(antiSsrfService).validateHostWithoutCache("smtp.example.com");
    verify(emailManager).sendVerification(emailConfig, "baz", ADDRESS);
  }

  private static EmailConfigurationXO getConfigurationXO(final String password) {
    return getConfigurationXOWithHost("localhost", password);
  }

  private static EmailConfigurationXO getConfigurationXOWithHost(final String host, final String password) {
    return new EmailConfigurationXO(
        false, host, 25,
        "foo", password, "astrum@example.org",
        null, false, false,
        false, false, false);
  }
}
