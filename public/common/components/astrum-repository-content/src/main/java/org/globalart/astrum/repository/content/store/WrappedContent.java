
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.repository.content.RepositoryContent;

/**
 * {@link RepositoryContent} wrapper where the 'real' instance is wrapped behind an interface.
 *
 * @since 3.24
 */
public interface WrappedContent<T extends RepositoryContent>
{
  T unwrap();
}
