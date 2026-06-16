
package org.globalart.astrum.coreui.internal.wonderland;

import java.util.List;

import jakarta.ws.rs.GET;
import jakarta.ws.rs.Path;
import jakarta.ws.rs.Produces;

import org.globalart.astrum.common.property.SystemPropertiesHelper;
import org.globalart.astrum.rest.Resource;

import com.google.common.collect.Lists;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML;
import org.springframework.stereotype.Component;

/**
 * Expose Astrum settings.
 *
 * @since 2.7
 */
@Component
@Path(SettingsResource.RESOURCE_URI)
public class SettingsResource
    implements Resource
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String RESOURCE_URI = "/wonderland/settings";

  @GET
  @Produces({APPLICATION_JSON, APPLICATION_XML})
  public List<PropertyXO> get() {
    List<PropertyXO> properties = Lists.newArrayList();

    properties.add(
        new PropertyXO().withKey("keepAlive")
            .withValue(Boolean.toString(SystemPropertiesHelper.getBoolean("astrum.ui.keepAlive", true))));

    return properties;
  }
}
