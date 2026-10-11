package org.globalart.astrum.coreui.internal.recovery;

import org.globalart.astrum.scheduling.RecoveryModeService;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.beans.factory.ObjectProvider;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests {@link RecoveryModeStateContributor}.
 */
@ExtendWith(MockitoExtension.class)
class RecoveryModeStateContributorTest
{
  @Mock
  private ObjectProvider<RecoveryModeService> recoveryModeService;

  @Test
  void recoveryModeIsAvailableWhenAPluginProvidesIt() {
    when(recoveryModeService.getIfAvailable()).thenReturn(mock(RecoveryModeService.class));

    assertThat(new RecoveryModeStateContributor(recoveryModeService).getState(),
        hasEntry(RecoveryModeStateContributor.RECOVERY_MODE_AVAILABLE, (Object) true));
  }

  @Test
  void recoveryModeIsNotAvailableWithoutTheService() {
    when(recoveryModeService.getIfAvailable()).thenReturn(null);

    assertThat(new RecoveryModeStateContributor(recoveryModeService).getState(),
        hasEntry(RecoveryModeStateContributor.RECOVERY_MODE_AVAILABLE, (Object) false));
  }
}
