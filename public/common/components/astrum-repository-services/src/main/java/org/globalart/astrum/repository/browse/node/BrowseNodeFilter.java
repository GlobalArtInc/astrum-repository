
package org.globalart.astrum.repository.browse.node;

import java.util.function.BiPredicate;

/**
 * This interface is used to filter the list of browse nodes that are returned at a given level of the browse tree
 * and we need to be very careful to ensure this condition is not too expensive. It is also worth noting that this has
 * the ability to hide non-leaf nodes which if implemented incorrectly could hide entire branches.
 *
 * @since 3.11
 */
public interface BrowseNodeFilter
    extends BiPredicate<BrowseNode, Boolean>
{
}
