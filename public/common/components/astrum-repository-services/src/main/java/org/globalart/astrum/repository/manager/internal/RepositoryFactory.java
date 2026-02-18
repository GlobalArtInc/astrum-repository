
package org.globalart.astrum.repository.manager.internal;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;

/**
 * Repository factory.
 *
 * @since 3.0
 */
public interface RepositoryFactory
{
  Repository create(Type type, Format format);
}
