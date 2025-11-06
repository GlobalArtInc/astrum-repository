
package org.globalart.astrum.bootstrap.spring;

import org.springframework.context.ApplicationEvent;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * This event is fired after Astrum is done performing its component scan. The flow is as follows Spring does its
 * component scan, using the typical spring @CompnentScan annoation(s) or via packages set in the @SpringBootApplication
 * annotations
 */
public class AstrumComponentScanCompleteEvent
    extends ApplicationEvent
{
  public AstrumComponentScanCompleteEvent(final ConfigurableApplicationContext configurableApplicationContext) {
    super(configurableApplicationContext);
  }
}
