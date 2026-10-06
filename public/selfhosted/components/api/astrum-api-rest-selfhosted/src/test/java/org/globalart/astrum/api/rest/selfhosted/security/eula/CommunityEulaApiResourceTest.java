
package org.globalart.astrum.api.rest.selfhosted.security.eula;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.api.rest.selfhosted.security.eula.model.EulaStatus;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class CommunityEulaApiResourceTest
{
  @Mock
  private GlobalKeyValueStore mockGlobalKeyValueStore;

  @InjectMocks
  private CommunityEulaApiResource underTest;

  @Test
  void testGetCommunityEulaStatusNotSet() {
    when(mockGlobalKeyValueStore.getKey(anyString())).thenReturn(Optional.empty());
    EulaStatus eulaStatus = underTest.getCommunityEulaStatus();
    assertFalse(eulaStatus.isAccepted());
  }

  @Test
  void testGetCommunityEulaStatusSet() {
    AstrumKeyValue keyValue = new AstrumKeyValue();
    keyValue.setValue(Map.of("accepted", true));
    when(mockGlobalKeyValueStore.getKey(anyString())).thenReturn(Optional.of(keyValue));
    EulaStatus eulaStatus = underTest.getCommunityEulaStatus();
    assertTrue(eulaStatus.isAccepted());
  }

  @Test
  void testSetEulaAccepted() {
    EulaStatus eulaStatus = new EulaStatus();
    eulaStatus.setAccepted(true);
    eulaStatus.setDisclaimer(EulaStatus.EXPECTED_DISCLAIMER);
    underTest.setEulaAcceptedCE(eulaStatus);
    verify(mockGlobalKeyValueStore, times(1)).setKey(any(AstrumKeyValue.class));
  }

  @Test
  void testsAttemptToUnacceptEula() {
    AstrumKeyValue keyValue = new AstrumKeyValue();
    keyValue.setValue(Map.of("accepted", true));
    when(mockGlobalKeyValueStore.getKey(anyString())).thenReturn(Optional.of(keyValue));
    // Attempt to set EULA as not accepted
    EulaStatus eulaStatus = new EulaStatus();
    eulaStatus.setAccepted(false);
    eulaStatus.setDisclaimer(EulaStatus.EXPECTED_DISCLAIMER);
    IllegalArgumentException thrown = assertThrows(IllegalArgumentException.class, () -> {
      underTest.setEulaAcceptedCE(eulaStatus);
    });

    assertEquals("EULA must be accepted", thrown.getMessage());
    verify(mockGlobalKeyValueStore, times(0)).setKey(any(AstrumKeyValue.class));
  }

  @Test
  void testReacceptEula() {
    AstrumKeyValue keyValue = new AstrumKeyValue();
    keyValue.setValue(Map.of("accepted", true));
    when(mockGlobalKeyValueStore.getKey(anyString())).thenReturn(Optional.of(keyValue));
    // Accept again while previously in an accepted state
    EulaStatus eulaStatus = new EulaStatus();
    eulaStatus.setAccepted(true);
    eulaStatus.setDisclaimer(EulaStatus.EXPECTED_DISCLAIMER);
    underTest.setEulaAcceptedCE(eulaStatus);
    verify(mockGlobalKeyValueStore, times(0)).setKey(any(AstrumKeyValue.class));
  }

  @Test
  void testInvalidDisclaimer() {
    EulaStatus eulaStatus = new EulaStatus();
    eulaStatus.setAccepted(true);
    eulaStatus.setDisclaimer("foo");
    IllegalArgumentException thrown = assertThrows(IllegalArgumentException.class, () -> {
      underTest.setEulaAcceptedCE(eulaStatus);
    });

    assertEquals("Invalid EULA disclaimer", thrown.getMessage());
    verify(mockGlobalKeyValueStore, times(0)).setKey(any(AstrumKeyValue.class));
  }
}
