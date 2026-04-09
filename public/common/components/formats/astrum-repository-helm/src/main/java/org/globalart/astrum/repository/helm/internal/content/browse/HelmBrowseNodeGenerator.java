
package org.globalart.astrum.repository.helm.internal.content.browse;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.content.browse.DefaultBrowseNodeGenerator;
import org.globalart.astrum.repository.helm.internal.HelmFormat;

/**
 * Helm places components one level above their assets.
 *
 * @since 3.28
 */
@Component
@Qualifier(HelmFormat.NAME)
public class HelmBrowseNodeGenerator
    extends DefaultBrowseNodeGenerator
{
}
