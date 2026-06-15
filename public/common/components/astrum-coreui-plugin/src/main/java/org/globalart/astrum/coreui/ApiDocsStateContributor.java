
package org.globalart.astrum.coreui;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * API feature flag
 */
@Component
public class ApiDocsStateContributor
    implements StateContributor
{
  private final boolean enabled;

  @Autowired
  public ApiDocsStateContributor(
      @Value("${astrum.admin.system.apidocs.enabled:true}") final boolean enabled)
  {
    this.enabled = enabled;
  }

  @Override
  @Nullable
  public Map<String, Object> getState() {
    return ImmutableMap.of("api", enabled);
  }
}
