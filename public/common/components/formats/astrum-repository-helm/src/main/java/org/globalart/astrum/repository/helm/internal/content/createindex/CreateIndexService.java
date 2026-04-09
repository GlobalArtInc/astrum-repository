
package org.globalart.astrum.repository.helm.internal.content.createindex;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Content;

/**
 * Build index.yaml file for Helm Hosted
 *
 * @since 3.28
 */
public interface CreateIndexService
{
  Content buildIndexYaml(final Repository repository);
}
