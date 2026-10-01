
package org.globalart.astrum.api.extdirect.selfhosted.atlas;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.atlas.SystemInformationGenerator;
import org.globalart.astrum.extdirect.DirectComponent;
import org.globalart.astrum.extdirect.DirectComponentSupport;

import com.codahale.metrics.annotation.ExceptionMetered;
import com.codahale.metrics.annotation.Timed;
import com.softwarementors.extjs.djn.config.annotations.DirectAction;
import com.softwarementors.extjs.djn.config.annotations.DirectMethod;
import org.apache.shiro.authz.annotation.RequiresPermissions;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * System Information {@link DirectComponent}.
 */
@Component
@DirectAction(action = "atlas_SystemInformation")
public class SystemInformationComponent
    extends DirectComponentSupport
{
  private final SystemInformationGenerator systemInformationGenerator;

  @Autowired
  public SystemInformationComponent(final SystemInformationGenerator systemInformationGenerator) {
    this.systemInformationGenerator = checkNotNull(systemInformationGenerator);
  }

  /**
   * Retrieves system information.
   *
   * @return a tree-structured report of critical system information details
   */
  @DirectMethod
  @Timed
  @ExceptionMetered
  @RequiresPermissions("astrum:atlas:read")
  public Map<String, Object> read() {
    return systemInformationGenerator.report();
  }
}
