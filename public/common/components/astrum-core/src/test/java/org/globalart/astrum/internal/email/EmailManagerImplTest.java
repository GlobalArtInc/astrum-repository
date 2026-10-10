
package org.globalart.astrum.internal.email;

import org.apache.commons.mail2.core.EmailConstants;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.email.EmailConfiguration;
import org.globalart.astrum.security.UserIdHelper;
import org.globalart.astrum.ssl.TrustStore;

import org.apache.commons.mail2.jakarta.Email;
import org.apache.commons.mail2.jakarta.SimpleEmail;
import org.mockito.MockedStatic;

import jakarta.inject.Provider;
import javax.net.ssl.SSLContext;
import java.util.function.Function;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.*;
import static org.junit.Assert.assertThrows;
import static org.mockito.Mockito.*;
import static org.globalart.astrum.internal.email.EmailManagerImpl.EMAIL_CONFIGURATION_SOURCE;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class EmailManagerImplTest
{
  private static final String PASSWORD_PLACEHOLDER = "#~ASTRUM~PLACEHOLDER~PASSWORD~#";

  @Mock
  private EventManager eventManager;

  @Mock
  private EmailConfigurationStore emailConfigurationStore;

  @Mock
  private TrustStore trustStore;

  @Mock
  private Function<EmailConfiguration, EmailConfiguration> defaults;

  @Mock
  private Provider capabilityRegistryProvider;

  @Mock
  private SecretsService secretsService;

  @InjectMocks
  private EmailManagerImpl emailManager;

  private MockedStatic<UserIdHelper> userIdHelperMock;

  @Before
  public void setup() {
    userIdHelperMock = mockStatic(UserIdHelper.class);
    userIdHelperMock.when(UserIdHelper::get).thenReturn("userId");
  }

  @After
  public void tearDown() {
    userIdHelperMock.close();
  }

  @Test
  public void testConfiguresEmailsCorrectly() throws Exception {
    when(trustStore.getSSLContext()).thenReturn(SSLContext.getDefault());

    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.getHost()).thenReturn("example.com");
    when(emailConfig.getPort()).thenReturn(25);
    when(emailConfig.getFromAddress()).thenReturn("sender@example.com");
    when(emailConfig.getUsername()).thenReturn("user");
    when(emailConfig.isStartTlsEnabled()).thenReturn(true);
    when(emailConfig.isStartTlsRequired()).thenReturn(false);
    when(emailConfig.isSslOnConnectEnabled()).thenReturn(false);
    when(emailConfig.isSslCheckServerIdentityEnabled()).thenReturn(false);
    when(emailConfig.isAstrumTrustStoreEnabled()).thenReturn(true);

    SimpleEmail mail = spy(SimpleEmail.class);

    Email email = emailManager.apply(emailConfig, mail, "pass");

    assertThat(email.isStartTLSEnabled(), is(true));
    assertThat(email.isStartTLSRequired(), is(false));
    assertThat(email.isSSLOnConnect(), is(false));
    assertThat(email.isSSLCheckServerIdentity(), is(false));
    assertThat(email.getMailSession().getProperties().containsKey("mail.smtp.ssl.socketFactory"), is(true));
    // MAIL_SMTP_SSL_ENABLE should not be set when using STARTTLS (only for SSL on connect)
    assertThat(email.getMailSession().getProperties().containsKey(EmailConstants.MAIL_SMTP_SSL_ENABLE), is(false));
    assertThat(email.getMailSession().getProperties().containsKey("mail.smtp.socketFactory.class"), is(false));
    verify(mail).setAuthentication("user", "pass");
  }

  @Test
  public void testSetConfiguration() {
    EmailConfiguration oldEmailConfig = mock(EmailConfiguration.class);
    when(emailConfigurationStore.load()).thenReturn(oldEmailConfig);
    when(oldEmailConfig.copy()).thenReturn(oldEmailConfig);
    Secret oldPass = mock(Secret.class);
    when(oldEmailConfig.getPassword()).thenReturn(oldPass);
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    Secret secret = mock(Secret.class);
    when(emailConfig.copy()).thenReturn(emailConfig);
    when(secretsService.encrypt(anyString(), any(), anyString())).thenReturn(secret);

    emailManager.setConfiguration(emailConfig, "newPassword");

    verify(emailConfigurationStore).save(emailConfig);
    verify(secretsService).encrypt(EMAIL_CONFIGURATION_SOURCE, "newPassword".toCharArray(), "userId");
    verify(secretsService).remove(oldPass);
    verify(eventManager).post(any());
  }

  @Test
  public void testSetConfigurationWithEmptyPassword() {
    EmailConfiguration oldEmailConfig = mock(EmailConfiguration.class);
    when(emailConfigurationStore.load()).thenReturn(oldEmailConfig);
    when(oldEmailConfig.copy()).thenReturn(oldEmailConfig);
    Secret oldPass = mock(Secret.class);
    when(oldEmailConfig.getPassword()).thenReturn(oldPass);
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    Secret secret = mock(Secret.class);
    when(emailConfig.copy()).thenReturn(emailConfig);
    when(secretsService.encrypt(anyString(), any(), anyString())).thenReturn(secret);

    emailManager.setConfiguration(emailConfig, "");

    verify(emailConfigurationStore).save(emailConfig);
    verify(secretsService, never()).encrypt(anyString(), any(), anyString());
    verify(secretsService).remove(oldPass);
    verify(eventManager).post(any());
  }

  @Test
  public void testSetConfigurationWithException() {
    EmailConfiguration oldEmailConfig = mock(EmailConfiguration.class);
    when(emailConfigurationStore.load()).thenReturn(oldEmailConfig);
    when(oldEmailConfig.copy()).thenReturn(oldEmailConfig);
    Secret oldPass = mock(Secret.class);
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    Secret secret = mock(Secret.class);
    when(emailConfig.copy()).thenReturn(emailConfig);
    when(secretsService.encrypt(anyString(), any(), anyString())).thenReturn(secret);
    when(emailConfig.getPassword()).thenReturn(secret);
    doThrow(new RuntimeException()).when(emailConfigurationStore).save(emailConfig);

    assertThrows(RuntimeException.class, () -> emailManager.setConfiguration(emailConfig, "newPassword"));

    verify(secretsService).encrypt(EMAIL_CONFIGURATION_SOURCE, "newPassword".toCharArray(), "userId");
    verify(secretsService).remove(secret);
    verify(secretsService, never()).remove(oldPass);
  }

  @Test
  public void testSetConfigurationWithPlaceHolder() {
    EmailConfiguration oldEmailConfig = mock(EmailConfiguration.class);
    when(emailConfigurationStore.load()).thenReturn(oldEmailConfig);
    when(oldEmailConfig.copy()).thenReturn(oldEmailConfig);
    Secret oldPass = mock(Secret.class);
    when(oldEmailConfig.getPassword()).thenReturn(oldPass);
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.copy()).thenReturn(emailConfig);
    when(emailConfig.getPassword()).thenReturn(oldPass);

    emailManager.setConfiguration(emailConfig, PASSWORD_PLACEHOLDER);

    verify(emailConfigurationStore).save(emailConfig);
    verify(secretsService, never()).encrypt(EMAIL_CONFIGURATION_SOURCE, PASSWORD_PLACEHOLDER.toCharArray(), "userId");
    verify(secretsService, never()).remove(oldPass);
    verify(eventManager).post(any());
  }

  @Test
  public void testSendWithEnabledConfiguration() throws Exception {
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.isEnabled()).thenReturn(true);
    when(emailConfigurationStore.load()).thenReturn(emailConfig);
    when(trustStore.getSSLContext()).thenReturn(SSLContext.getDefault());

    Email email = mock(Email.class);

    emailManager.send(email);

    verify(email).send();
  }

  @Test
  public void testSendWithDisabledConfiguration() throws Exception {
    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.isEnabled()).thenReturn(false);
    when(emailConfigurationStore.load()).thenReturn(emailConfig);

    Email email = mock(Email.class);

    emailManager.send(email);

    verify(email, never()).send();
  }

  @Test
  public void testSslOnConnectWithTrustStoreEnabled() throws Exception {
    when(trustStore.getSSLContext()).thenReturn(SSLContext.getDefault());

    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.getHost()).thenReturn("example.com");
    when(emailConfig.getPort()).thenReturn(465);
    when(emailConfig.getFromAddress()).thenReturn("sender@example.com");
    when(emailConfig.getUsername()).thenReturn("user");
    when(emailConfig.isStartTlsEnabled()).thenReturn(false);
    when(emailConfig.isStartTlsRequired()).thenReturn(false);
    when(emailConfig.isSslOnConnectEnabled()).thenReturn(true);
    when(emailConfig.isSslCheckServerIdentityEnabled()).thenReturn(true);
    when(emailConfig.isAstrumTrustStoreEnabled()).thenReturn(true);

    SimpleEmail mail = spy(SimpleEmail.class);

    Email email = emailManager.apply(emailConfig, mail, "pass");

    assertThat(email.isStartTLSEnabled(), is(false));
    assertThat(email.isStartTLSRequired(), is(false));
    assertThat(email.isSSLOnConnect(), is(true));
    assertThat(email.isSSLCheckServerIdentity(), is(true));
    // SSL socket factory and MAIL_SMTP_SSL_ENABLE should be set for SSL on connect
    assertThat(email.getMailSession().getProperties().containsKey("mail.smtp.ssl.socketFactory"), is(true));
    assertThat(email.getMailSession().getProperties().containsKey(EmailConstants.MAIL_SMTP_SSL_ENABLE), is(true));
    assertThat(email.getMailSession().getProperties().get(EmailConstants.MAIL_SMTP_SSL_ENABLE), is(true));
    verify(mail).setAuthentication("user", "pass");
  }

  @Test
  public void testNonSslEmailServerWithTrustStoreEnabled() throws Exception {
    when(trustStore.getSSLContext()).thenReturn(SSLContext.getDefault());

    EmailConfiguration emailConfig = mock(EmailConfiguration.class);
    when(emailConfig.getHost()).thenReturn("example.com");
    when(emailConfig.getPort()).thenReturn(25);
    when(emailConfig.getFromAddress()).thenReturn("sender@example.com");
    when(emailConfig.getUsername()).thenReturn("user");
    when(emailConfig.isStartTlsEnabled()).thenReturn(false);
    when(emailConfig.isStartTlsRequired()).thenReturn(false);
    when(emailConfig.isSslOnConnectEnabled()).thenReturn(false);
    when(emailConfig.isSslCheckServerIdentityEnabled()).thenReturn(false);
    when(emailConfig.isAstrumTrustStoreEnabled()).thenReturn(true);

    SimpleEmail mail = spy(SimpleEmail.class);

    Email email = emailManager.apply(emailConfig, mail, "pass");

    assertThat(email.isStartTLSEnabled(), is(false));
    assertThat(email.isStartTLSRequired(), is(false));
    assertThat(email.isSSLOnConnect(), is(false));
    assertThat(email.isSSLCheckServerIdentity(), is(false));
    // SSL socket factory should NOT be set because neither SSL nor STARTTLS is enabled
    assertThat(email.getMailSession().getProperties().containsKey("mail.smtp.ssl.socketFactory"), is(false));
    assertThat(email.getMailSession().getProperties().containsKey(EmailConstants.MAIL_SMTP_SSL_ENABLE), is(false));
    verify(mail).setAuthentication("user", "pass");
  }

  @Test
  public void testOnStoreChanged() {
    EmailConfigurationData newConfig = new EmailConfigurationData();
    newConfig.setEnabled(true);
    newConfig.setHost("smtp.example.com");
    newConfig.setPort(587);
    newConfig.setUsername("user@example.com");
    Secret password = mock(Secret.class);
    newConfig.setPassword(password);
    newConfig.setFromAddress("noreply@example.com");
    newConfig.setSubjectPrefix("[Test]");
    newConfig.setStartTlsEnabled(true);
    newConfig.setStartTlsRequired(false);
    newConfig.setSslOnConnectEnabled(true);
    newConfig.setSslCheckServerIdentityEnabled(true);
    newConfig.setAstrumTrustStoreEnabled(false);

    when(emailConfigurationStore.load()).thenReturn(newConfig);

    EmailConfigurationChanged event = new EmailConfigurationChanged();
    event.setRemoteNodeId("remoteNodeId");
    emailManager.onStoreChanged(event);

    EmailConfiguration retrievedConfig = emailManager.getConfiguration();

    assertThat(retrievedConfig.isEnabled(), is(true));
    assertThat(retrievedConfig.getHost(), is("smtp.example.com"));
    assertThat(retrievedConfig.getPort(), is(587));
    assertThat(retrievedConfig.getUsername(), is("user@example.com"));
    assertThat(retrievedConfig.getPassword(), is(password));
    assertThat(retrievedConfig.getFromAddress(), is("noreply@example.com"));
    assertThat(retrievedConfig.getSubjectPrefix(), is("[Test]"));
    assertThat(retrievedConfig.isStartTlsEnabled(), is(true));
    assertThat(retrievedConfig.isStartTlsRequired(), is(false));
    assertThat(retrievedConfig.isSslOnConnectEnabled(), is(true));
    assertThat(retrievedConfig.isSslCheckServerIdentityEnabled(), is(true));
    assertThat(retrievedConfig.isAstrumTrustStoreEnabled(), is(false));
  }
}
