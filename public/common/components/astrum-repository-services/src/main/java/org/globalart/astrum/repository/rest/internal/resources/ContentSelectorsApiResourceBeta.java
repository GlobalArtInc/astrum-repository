
package org.globalart.astrum.repository.rest.internal.resources;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.ws.rs.Path;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.rest.APIConstants;
import org.globalart.astrum.security.internal.rest.SecurityApiConstants;
import org.globalart.astrum.selector.SelectorConfigurationStore;
import org.globalart.astrum.selector.SelectorFactory;
import org.globalart.astrum.selector.SelectorManager;

import io.swagger.v3.oas.annotations.Hidden;

import static org.globalart.astrum.repository.rest.internal.resources.ContentSelectorsApiResourceBeta.RESOURCE_URI;
import org.springframework.stereotype.Component;

/**
 * @since 3.26
 * @deprecated beta prefix is being phased out, prefer starting new APIs with {@link APIConstants#V1_API_PREFIX} instead
 */
@Hidden
@Component
@Path(RESOURCE_URI)
@Deprecated
public class ContentSelectorsApiResourceBeta
    extends ContentSelectorsApiResource
{
  static final String RESOURCE_URI = SecurityApiConstants.BETA_RESOURCE_URI + "content-selectors";

  @Autowired
  public ContentSelectorsApiResourceBeta(
      final SelectorFactory selectorFactory,
      final SelectorManager selectorManager,
      final SelectorConfigurationStore store,
      final EventManager eventManager)
  {
    super(selectorFactory, selectorManager, store, eventManager);
  }
}
