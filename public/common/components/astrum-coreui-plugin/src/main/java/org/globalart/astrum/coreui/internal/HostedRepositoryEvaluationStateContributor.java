
package org.globalart.astrum.coreui.internal;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Value;

import static org.globalart.astrum.common.app.FeatureFlags.HOSTED_REPOSITORY_EVALUATION_ENABLED_NAMED_VALUE;
import org.springframework.stereotype.Component;

@Component
public class HostedRepositoryEvaluationStateContributor
    implements StateContributor
{
  private final boolean isHostedRepositoryEvaluationEnabled;

  @Autowired
  public HostedRepositoryEvaluationStateContributor(
      @Value(HOSTED_REPOSITORY_EVALUATION_ENABLED_NAMED_VALUE) final boolean isHostedRepositoryEvaluationEnabled)
  {
    this.isHostedRepositoryEvaluationEnabled = isHostedRepositoryEvaluationEnabled;
  }

  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of("hostedRepositoryEvaluationEnabled", isHostedRepositoryEvaluationEnabled);
  }
}
