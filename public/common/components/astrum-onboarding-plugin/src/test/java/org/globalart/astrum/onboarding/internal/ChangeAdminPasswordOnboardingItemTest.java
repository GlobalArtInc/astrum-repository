
package org.globalart.astrum.onboarding.internal;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserStatus;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ChangeAdminPasswordOnboardingItemTest
{
  @Mock
  private SecuritySystem securitySystem;

  private ChangeAdminPasswordOnboardingItem underTest;

  @Before
  public void setup() {
    underTest = new ChangeAdminPasswordOnboardingItem(securitySystem);
  }

  @Test
  public void testApplies() throws Exception {
    User user = new User();
    user.setStatus(UserStatus.changepassword);

    when(securitySystem.getUser("admin", "default")).thenReturn(user);

    assertThat(underTest.applies(), is(true));
  }

  @Test
  public void testApplies_statusActive() throws Exception {
    User user = new User();
    user.setStatus(UserStatus.active);

    when(securitySystem.getUser("admin", "default")).thenReturn(user);

    assertThat(underTest.applies(), is(false));
  }

  @Test
  public void testApplies_statusDisabled() throws Exception {
    User user = new User();
    user.setStatus(UserStatus.disabled);

    when(securitySystem.getUser("admin", "default")).thenReturn(user);

    assertThat(underTest.applies(), is(false));
  }

  @Test
  public void testApplies_statusLocked() throws Exception {
    User user = new User();
    user.setStatus(UserStatus.locked);

    when(securitySystem.getUser("admin", "default")).thenReturn(user);

    assertThat(underTest.applies(), is(false));
  }

  @Test
  public void testApplies_userNotFound() throws Exception {
    when(securitySystem.getUser("admin", "default")).thenThrow(new UserNotFoundException("admin"));

    assertThat(underTest.applies(), is(false));
  }

  @Test
  public void testApplies_userManagerNotFound() throws Exception {
    when(securitySystem.getUser("admin", "default")).thenThrow(new NoSuchUserManagerException("default"));

    assertThat(underTest.applies(), is(false));
  }
}
