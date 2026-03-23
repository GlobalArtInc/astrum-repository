
package org.globalart.astrum.repository.maven.application.scan;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.application.scan.RepositoryApplicationScanSupport;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.HostedType;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(Maven2Format.NAME + "-" + HostedType.NAME)
public class Maven2HostedRepositoryApplicationScanSupport
    implements RepositoryApplicationScanSupport
{
  @Override
  public boolean isSupported() {
    return true;
  }

  @Override
  public String getStageTypeId(final Repository repository) {
    return RepositoryApplicationScanSupport.getStageTypeIdFromAttributes(repository, "maven", "versionPolicy");
  }
}
