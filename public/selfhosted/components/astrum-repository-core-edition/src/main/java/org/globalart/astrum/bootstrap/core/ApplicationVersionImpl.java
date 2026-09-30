
package org.globalart.astrum.bootstrap.core;

import org.globalart.astrum.bootstrap.entrypoint.edition.core.CoreAstrumEdition;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.ApplicationVersionSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

/**
 * CORE {@link ApplicationVersion}.
 */
@Component
@ConditionalOnProperty(value = "astrum.edition", havingValue = "CORE", matchIfMissing = true)
public class ApplicationVersionImpl
    extends ApplicationVersionSupport
{
  private final CoreAstrumEdition coreAstrumEdition;

  @Autowired
  public ApplicationVersionImpl(final CoreAstrumEdition coreAstrumEdition) {
    this.coreAstrumEdition = coreAstrumEdition;
  }

  @Override
  public String getEdition() {
    return coreAstrumEdition.getShortName();
  }
}
