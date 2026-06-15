
package org.globalart.astrum.coreui.internal.log;

import java.util.Collections;
import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
public class LogViewerStateContributor
    implements StateContributor
{
  private final boolean enabled;

  @Autowired
  public LogViewerStateContributor(@Value("${astrum.datastore.clustered.enabled:false}") final boolean enabled) {
    this.enabled = enabled;
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    return Collections.singletonMap("log.viewer.enabled", !enabled);
  }
}
