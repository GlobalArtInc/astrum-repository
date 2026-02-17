
package org.globalart.astrum.repository.browse.node;

/**
 * Customizes browse node distinctness for a particular format.
 *
 * @since 3.22
 */
public interface BrowseNodeIdentity
{
  String identity(BrowseNode browseNode);
}
