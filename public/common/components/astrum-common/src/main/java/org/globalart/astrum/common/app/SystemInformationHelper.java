
package org.globalart.astrum.common.app;

import java.util.Map;

import org.globalart.astrum.common.atlas.SystemInformationGenerator;

/**
 * Helper interface to populate system information {@link SystemInformationGenerator}
 *
 */
public interface SystemInformationHelper
{
  Map<String, Object> getValue();
}
