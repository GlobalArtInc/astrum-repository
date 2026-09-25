
package org.globalart.astrum.onboarding.internal;

import org.globalart.astrum.onboarding.capability.OnboardingCapability;
import org.globalart.astrum.onboarding.capability.OnboardingCapabilityHelper;
import org.globalart.astrum.security.anonymous.AnonymousManager;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class InstanceStatusTest
{
  private InstanceStatus underTest;

  @Mock
  private AnonymousManager anonymousManager;

  @Mock
  private OnboardingCapabilityHelper onboardingCapabilityHelper;

  @Mock
  private OnboardingCapability onboardingCapability;

  @Before
  public void setup() {
    underTest = new InstanceStatus(anonymousManager, onboardingCapabilityHelper);
    when(onboardingCapabilityHelper.getOnboardingCapability()).thenReturn(onboardingCapability);
  }

  @Test
  public void test_instanceIsNew_whenAnonymousNotConfigured() {
    when(anonymousManager.isConfigured()).thenReturn(false);

    assertThat(underTest.isNew(), is(true));
    assertThat(underTest.isUpgraded(), is(false));
  }

  @Test
  public void test_instanceIsUpgraded_whenAnonymousConfiguredButRegistrationNotStarted() {
    when(anonymousManager.isConfigured()).thenReturn(true);
    when(onboardingCapability.isRegistrationStarted()).thenReturn(false);

    assertThat(underTest.isNew(), is(false));
    assertThat(underTest.isUpgraded(), is(true));
  }

  @Test
  public void test_instanceIsNew_whenAnonymousConfiguredAndRegistrationStartedButNotCompleted() {
    when(anonymousManager.isConfigured()).thenReturn(true);
    when(onboardingCapability.isRegistrationStarted()).thenReturn(true);
    when(onboardingCapability.isRegistrationCompleted()).thenReturn(false);

    assertThat(underTest.isNew(), is(true));
    assertThat(underTest.isUpgraded(), is(false));
  }
}
