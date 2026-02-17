
package org.globalart.astrum.repository.application.scan;

import org.globalart.astrum.repository.Repository;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DefaultRepositoryApplicationScanSupport.DEFAULT)
public class DefaultRepositoryApplicationScanSupport
    implements RepositoryApplicationScanSupport
{
  public static final String DEFAULT = "default";

  @Override
  public boolean isSupported() {
    return false;
  }

  @Override
  public String getStageTypeId(final Repository repository) {
    return "stage-release";
  }
}
