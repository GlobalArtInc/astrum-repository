
package org.globalart.astrum.common.cooperation2.datastore.internal;

import org.globalart.astrum.common.cooperation2.Config;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.cooperation2.ScopedCooperation2Support;

/**
 * An implementation of {@link Cooperation2Factory} which uses local concurrency controls.
 *
 * @since 3.41
 */
public class LocalCooperation2
    extends ScopedCooperation2Support
{
  public LocalCooperation2(final String scope, final Config config) {
    super(scope, config);
  }
}
