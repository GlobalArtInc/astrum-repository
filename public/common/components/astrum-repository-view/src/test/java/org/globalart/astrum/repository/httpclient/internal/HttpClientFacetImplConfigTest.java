
package org.globalart.astrum.repository.httpclient.internal;

import java.util.Collections;
import java.util.Set;

import jakarta.validation.ConstraintViolation;
import jakarta.validation.Validation;
import jakarta.validation.Validator;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.httpclient.HttpClientManager;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatus;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatusEvent;
import org.globalart.astrum.repository.httpclient.HttpClientConfig;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.AVAILABLE;
import static org.globalart.astrum.repository.httpclient.RemoteConnectionStatusType.BLOCKED;

/**
 * Tests for {@link HttpClientFacetImpl.Config}.
 */
public class HttpClientFacetImplConfigTest
{
  private Validator validator;

  private EventManager eventManager = mock(EventManager.class);

  @Before
  public void setUp() throws Exception {
    validator = Validation.buildDefaultValidatorFactory().getValidator();
  }

  @Test
  public void authenticationUsernameWithNullPassword() {
    UsernameAuthenticationConfiguration auth = new UsernameAuthenticationConfiguration();
    auth.setUsername("admin");
    auth.setPassword(null);
    HttpClientConfig config = new HttpClientConfig();
    config.authentication = auth;
    Set<ConstraintViolation<HttpClientConfig>> violations = validator.validate(config);
    assertThat(violations.size(), equalTo(1));
    ConstraintViolation<HttpClientConfig> violation = violations.iterator().next();
    assertThat(violation.getPropertyPath().toString(), equalTo("authentication.password"));
  }

  @Test
  public void authenticationPasswordWithNullUsername() {
    UsernameAuthenticationConfiguration auth = new UsernameAuthenticationConfiguration();
    auth.setUsername("");
    auth.setPassword(mock(Secret.class));
    HttpClientConfig config = new HttpClientConfig();
    config.authentication = auth;
    Set<ConstraintViolation<HttpClientConfig>> violations = validator.validate(config);
    assertThat(violations.size(), equalTo(1));
    ConstraintViolation<HttpClientConfig> violation = violations.iterator().next();
    assertThat(violation.getPropertyPath().toString(), equalTo("authentication.username"));

  }

  @Test
  public void requiredFieldsMayNotBeWhitespaceOnly() {
    UsernameAuthenticationConfiguration auth = new UsernameAuthenticationConfiguration();
    auth.setUsername("");
    auth.setPassword(null);
    HttpClientConfig config = new HttpClientConfig();
    config.authentication = auth;
    Set<ConstraintViolation<HttpClientConfig>> violations = validator.validate(config);
    assertThat(violations.size(), equalTo(2));
    assertThat(violations.stream().map(v -> v.getPropertyPath().toString()).sorted().toArray(),
        equalTo(new String[]{"authentication.password", "authentication.username"}));
  }

  @Test
  public void fireEventOnRemoteConnectionStatusChanged() throws Exception {
    HttpClientFacetImpl underTest = new HttpClientFacetImpl(mock(HttpClientManager.class), Collections.emptyList(),
        Collections.emptyList(), Collections.emptyList(), Collections.emptyList(), Collections.emptyList());
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn("test-repository");
    underTest.attach(repository);
    underTest.installDependencies(eventManager);
    underTest.onStatusChanged(new RemoteConnectionStatus(AVAILABLE), new RemoteConnectionStatus(BLOCKED));
    verify(eventManager).post(any(RemoteConnectionStatusEvent.class));
  }
}
