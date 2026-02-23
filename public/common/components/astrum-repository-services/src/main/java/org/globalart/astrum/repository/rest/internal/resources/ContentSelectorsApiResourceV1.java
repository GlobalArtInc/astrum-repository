
package org.globalart.astrum.repository.rest.internal.resources;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.SelectorManager;

import static org.globalart.astrum.repository.rest.internal.resources.ContentSelectorsApiResourceV1.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 */
@Component
@Path(RESOURCE_URI)
public class ContentSelectorsApiResourceV1
    extends ContentSelectorsApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.V1_RESOURCE_URI + "content-selectors";

  @Autowired
  public ContentSelectorsApiResourceV1(
      final SelectorFactory selectorFactory,
      final SelectorManager selectorManager,
      final SelectorConfigurationStore store,
      final EventManager eventManager)
  {
    super(selectorFactory, selectorManager, store, eventManager);
  }
}
