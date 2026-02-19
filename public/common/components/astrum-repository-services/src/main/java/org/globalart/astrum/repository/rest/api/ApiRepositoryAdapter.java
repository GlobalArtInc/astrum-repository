
package org.globalart.astrum.repository.rest.api;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.FirewallAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiProxyRepository;

/**
 * Formats can implement this interface to supply custom objects to use in the RepositoryApiResource.
 *
 * @since 3.20
 */
public interface ApiRepositoryAdapter
{
  AbstractApiRepository adapt(Repository repository);

  /**
   * Wraps {@link #adapt} so cross-format properties can be applied uniformly without every
   * per-format adapter having to thread them through its own constructors. Currently used to
   * populate {@link FirewallAttributes} on proxy repositories; future cross-format properties
   * should be added here too.
   * <p>
   * REST resources that present API repository representations to clients should call this
   * method instead of {@link #adapt} directly.
   */
  default AbstractApiRepository adaptDecorated(final Repository repository) {
    AbstractApiRepository result = adapt(repository);
    if (result instanceof SimpleApiProxyRepository proxy) {
      proxy.setFirewall(FirewallAttributes.fromConfiguration(repository.getConfiguration()));
    }
    return result;
  }
}
