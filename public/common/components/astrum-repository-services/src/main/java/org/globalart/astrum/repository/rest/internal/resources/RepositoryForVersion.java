
package org.globalart.astrum.repository.rest.internal.resources;

/**
 * One row in the response of the "repositories containing a component version" endpoint.
 *
 * @param repositoryName the repository containing the selected version
 * @param type repository type value: "hosted", "proxy", or "group"
 * @param versionCount component-wide distinct-version count in this repository
 *          (across all versions of the component, not just the selected one)
 */
public record RepositoryForVersion(
    String repositoryName,
    String type,
    long versionCount)
{
}
