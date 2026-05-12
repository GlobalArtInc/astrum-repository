
package org.globalart.astrum.rapture.internal.state;

import java.util.Collection;
import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.systemchecks.NodeSystemCheckResult;
import org.globalart.astrum.systemchecks.SystemCheckService;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.ImmutableMap;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * State contributor that indicates if any of the system Health checks are currently failing
 *
 * @since 3.16
 */
@Component
public class HealthCheckStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @VisibleForTesting
  protected static final String HC_FAILED_KEY = "health_checks_failed";

  private SystemCheckService systemCheckService;

  @Autowired
  public HealthCheckStateContributor(final SystemCheckService systemCheckService) {
    this.systemCheckService = checkNotNull(systemCheckService);
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {

    boolean failed = systemCheckService.getResults()
        .map(NodeSystemCheckResult::getResult)
        .map(Map::values)
        .flatMap(Collection::stream)
        .anyMatch(result -> !result.isHealthy());

    return ImmutableMap.of(HC_FAILED_KEY, failed);
  }
}
