
package org.globalart.astrum.repository.search;

import java.time.OffsetDateTime;
import java.util.List;

/**
 * One distinct version of a component and the repositories containing it.
 */
public record ComponentVersion(String version, OffsetDateTime lastUpdated, List<String> repositories)
{
}
