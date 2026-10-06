
package org.globalart.astrum.self.hosted.state;

import java.util.Map;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.rapture.internal.state.StatusXO;

import com.google.common.collect.ImmutableMap;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Contributes {@code status} state.
 *
 * @since 3.0
 */
@Component
public class StatusStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String STATE_ID = "status";

  private final ApplicationVersion applicationVersion;

  @Autowired
  public StatusStateContributor(final ApplicationVersion applicationVersion) {
    this.applicationVersion = checkNotNull(applicationVersion);
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(STATE_ID, calculateStatus());
  }

  private Object calculateStatus() {
    StatusXO result = new StatusXO();
    result.setVersion(applicationVersion.getVersion());
    result.setEdition(applicationVersion.getEdition());
    result.setBuildRevision(applicationVersion.getBuildRevision());
    result.setBuildTimestamp(applicationVersion.getBuildTimestamp());
    return result;
  }
}
