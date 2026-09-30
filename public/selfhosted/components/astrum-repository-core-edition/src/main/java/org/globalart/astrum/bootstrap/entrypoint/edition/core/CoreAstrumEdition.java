
package org.globalart.astrum.bootstrap.entrypoint.edition.core;

import org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionSupport;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.bootstrap.entrypoint.edition.AstrumEditionPriority.CORE_PRIORITY;

@Component
public class CoreAstrumEdition
    extends AstrumEditionSupport
{
  public static final String NAME = "astrum-core-edition";

  public static final String SHORT_NAME = "CORE";

  public CoreAstrumEdition() {
    super(NAME, NAME, SHORT_NAME, CORE_PRIORITY);
  }

  @Override
  public boolean isActive() {
    // no checks to do, if no other edition is enabled, core is enabled
    return true;
  }
}
