
package org.globalart.astrum.repository.rest.internal.resources;

import java.util.List;

/**
 * Response body for GET /service/rest/v1/search/repositories.
 *
 * @param items repositories containing the selected version, sorted by repositoryName
 * @param totalCount items.size() — feeds the Repositories tab badge on the UI
 */
public record RepositoriesForVersionResponse(
    List<RepositoryForVersion> items,
    int totalCount)
{
}
