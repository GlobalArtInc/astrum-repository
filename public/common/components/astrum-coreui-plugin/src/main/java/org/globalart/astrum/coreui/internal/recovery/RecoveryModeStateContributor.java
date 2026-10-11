package org.globalart.astrum.coreui.internal.recovery;

import java.util.Map;

import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.scheduling.RecoveryModeService;

import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Tells the UI whether this server has recovery mode, so that it offers the Recovery mode page only where the page
 * works: the mode is provided by an optional plugin.
 */
@Component
public class RecoveryModeStateContributor
    implements StateContributor
{
  public static final String RECOVERY_MODE_AVAILABLE = "recoveryModeAvailable";

  private final ObjectProvider<RecoveryModeService> recoveryModeService;

  @Autowired
  public RecoveryModeStateContributor(final ObjectProvider<RecoveryModeService> recoveryModeService) {
    this.recoveryModeService = checkNotNull(recoveryModeService);
  }

  @Override
  public Map<String, Object> getState() {
    return Map.of(RECOVERY_MODE_AVAILABLE, recoveryModeService.getIfAvailable() != null);
  }
}
