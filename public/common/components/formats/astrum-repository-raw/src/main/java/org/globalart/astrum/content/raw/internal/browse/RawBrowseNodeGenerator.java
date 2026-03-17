
package org.globalart.astrum.content.raw.internal.browse;

import org.globalart.astrum.repository.content.browse.AssetPathBrowseNodeGenerator;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * RAW places components at the same level as their assets.
 *
 * @since 3.26
 */
@Component
@Qualifier(RawFormat.NAME)
public class RawBrowseNodeGenerator
    extends AssetPathBrowseNodeGenerator
{
}
