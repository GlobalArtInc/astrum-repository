
package org.globalart.astrum.security.anonymous;

import java.util.Arrays;
import java.util.List;

import org.globalart.astrum.security.internal.AuthorizingRealmImpl;
import org.globalart.astrum.security.internal.DefaultRealmConstants;
import org.globalart.astrum.security.user.UserManager;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasItem;
import static org.hamcrest.Matchers.not;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link AnonymousHelper}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AnonymousHelperTest
{
  private static final String TEST_REALM = "TEST";

  @Mock
  private UserManager userManager1;

  @Mock
  private UserManager userManagerAuth;

  @Before
  public void setup() {
    when(userManager1.getAuthenticationRealmName()).thenReturn(TEST_REALM);
    when(userManagerAuth.getAuthenticationRealmName()).thenReturn(DefaultRealmConstants.DEFAULT_REALM_NAME);
  }

  @Test
  public void testGetAuthenticationRealms() {
    List<UserManager> userManagers = Arrays.asList(userManager1, userManagerAuth);

    List<String> realms = AnonymousHelper.getAuthenticationRealms(userManagers);

    assertThat(realms, hasItem(userManager1.getAuthenticationRealmName()));
    assertThat(realms, not(hasItem(userManagerAuth.getAuthenticationRealmName())));
    assertThat(realms, hasItem(AuthorizingRealmImpl.NAME));
  }
}
