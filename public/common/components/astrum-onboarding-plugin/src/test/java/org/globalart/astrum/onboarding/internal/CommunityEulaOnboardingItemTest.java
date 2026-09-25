
package org.globalart.astrum.onboarding.internal;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;

import org.junit.Before;
import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class CommunityEulaOnboardingItemTest
{
  @Mock
  private ApplicationVersion mockApplicationVersion;

  @Mock
  private GlobalKeyValueStore mockGlobalKeyValueStore;

  @InjectMocks
  private CommunityEulaOnboardingItem underTest;

  @Before
  public void setUp() {
    when(mockApplicationVersion.getEdition()).thenReturn("COMMUNITY");
  }

  @Test
  public void testAppliesWhenCommunityAndEulaNotAccepted() {
    when(mockGlobalKeyValueStore.getKey("astrum.community.eula.accepted")).thenReturn(Optional.empty());
    assertTrue(underTest.applies());
  }

  @Test
  public void testAppliesWhenCommunityAndEulaAccepted() {
    AstrumKeyValue eulaStatus = new AstrumKeyValue();
    eulaStatus.setValue(Map.of("accepted", true));
    when(mockGlobalKeyValueStore.getKey("astrum.community.eula.accepted")).thenReturn(Optional.of(eulaStatus));
    assertFalse(underTest.applies());
  }

  @Test
  public void testAppliesWhenNotCommunity() {
    when(mockApplicationVersion.getEdition()).thenReturn("PRO");
    assertFalse(underTest.applies());
  }
}
