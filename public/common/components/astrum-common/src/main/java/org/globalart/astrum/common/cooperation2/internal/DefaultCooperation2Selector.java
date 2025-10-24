
package org.globalart.astrum.common.cooperation2.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.cooperation2.Cooperation2Selector;

import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

/**
 * Cooperation2Selector implementation that simply returns the default impl from the dependency injector.
 */
@Primary
@Component
public class DefaultCooperation2Selector
    implements Cooperation2Selector
{
  private Cooperation2Factory cooperation2Factory;

  @Autowired
  public DefaultCooperation2Selector(final Cooperation2Factory cooperation2Factory) {
    this.cooperation2Factory = cooperation2Factory;
  }

  @Override
  public Cooperation2Factory select() {
    return cooperation2Factory;
  }
}
