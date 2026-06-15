
package org.globalart.astrum.coreui.internal;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.rapture.StateContributor;

import com.google.common.collect.ImmutableMap;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class WorkDirectoryStateContributor
    implements StateContributor
{
  static final String WORK_DIRECTORY_KEY = "astrum.application.workDirectory";

  private final ApplicationDirectories applicationDirectories;

  @Autowired
  public WorkDirectoryStateContributor(final ApplicationDirectories applicationDirectories) {
    this.applicationDirectories = applicationDirectories;
  }

  @Nullable
  @Override
  public Map<String, Object> getState() {
    return ImmutableMap.of(WORK_DIRECTORY_KEY, applicationDirectories.getWorkDirectory().getAbsolutePath());
  }
}
