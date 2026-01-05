
package org.globalart.astrum.security.internal;

import java.util.Arrays;

import org.globalart.astrum.security.authc.AuthRateLimiterService;
import org.globalart.astrum.security.authc.UserPasswordChanged;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserUpdatedEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.verify;

@RunWith(MockitoJUnitRunner.class)
public class AuthRateLimiterResetListenerTest
{
  @Mock
  private AuthRateLimiterService rateLimiterService;

  private AuthRateLimiterResetListener listener;

  @Before
  public void setUp() {
    listener = new AuthRateLimiterResetListener(rateLimiterService);
  }

  @Test
  public void testClass_hasConditionalOnPropertyForEnabledFlag() {
    ConditionalOnProperty annotation = AuthRateLimiterResetListener.class.getAnnotation(ConditionalOnProperty.class);

    assertThat("@ConditionalOnProperty must be present", annotation, is(notNullValue()));
    assertThat(Arrays.asList(annotation.name()), hasItem("astrum.auth.ratelimit.enabled"));
    assertThat(annotation.havingValue(), is("true"));
    assertThat(annotation.matchIfMissing(), is(true));
  }

  @Test
  public void testOnUserUpdated_resetsRateLimitCounter() {
    User user = new User();
    user.setUserId("jsmith");

    listener.on(new UserUpdatedEvent(user));

    verify(rateLimiterService).reset("jsmith");
  }

  @Test
  public void testOnPasswordChanged_resetsRateLimitCounter() {
    listener.on(new UserPasswordChanged("jsmith"));

    verify(rateLimiterService).reset("jsmith");
  }
}
