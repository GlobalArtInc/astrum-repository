
package org.globalart.astrum.repository.pypi.internal;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.browse.node.BrowseNode;
import org.globalart.astrum.repository.browse.node.BrowseNodeIdentity;

/**
 * PyPi custom {@link BrowseNodeIdentity}
 *
 * @since 3.22
 */
@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiBrowseNodeIdentity
    implements BrowseNodeIdentity
{
  /**
   * Changes default behavior to be distinct and ignore case
   */
  @Override
  public String identity(final BrowseNode browseNode) {
    return browseNode.getName().toLowerCase();
  }
}
