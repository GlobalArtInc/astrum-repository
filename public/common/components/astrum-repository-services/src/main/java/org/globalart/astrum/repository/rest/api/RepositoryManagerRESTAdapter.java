
package org.globalart.astrum.repository.rest.api;

import java.util.List;
import java.util.Optional;
import java.util.Set;

import jakarta.ws.rs.WebApplicationException;

import org.globalart.astrum.repository.Repository;

/**
 * A component to share common functionality for interacting with {@link
 * org.globalart.astrum.repository.manager.RepositoryManager}.
 * between API resources
 *
 * @since 3.4
 */
public interface RepositoryManagerRESTAdapter
{
  /**
   * Retrieve a repository. Will throw a {@link jakarta.ws.rs.WebApplicationException} with status code 422 if the
   * supplied repository id is null, and throws a {@link jakarta.ws.rs.NotFoundException} if no repository with the
   * supplied id exists.
   */
  Repository toRepository(String repositoryId);

  /**
   * Retrieve a repository. Will throw a {@link jakarta.ws.rs.WebApplicationException} with status code 422 if the
   * supplied repository id is null and throws a {@link jakarta.ws.rs.NotFoundException} if no repository with the
   * supplied id exists.
   *
   * It throws a {@link WebApplicationException} with 403 status code if the repository exists and the user doesn't have
   * BROWSE permissions for it.
   */
  Repository getRepository(String repositoryId);

  /**
   * Retrieve a repository. Will throw a {@link jakarta.ws.rs.WebApplicationException} with status code 422 if the
   * supplied repository id is null, and throws a {@link jakarta.ws.rs.NotFoundException} if no repository with the
   * supplied id exists.
   *
   * Be careful ! Throws 403 if a user have not permissions(READ or BROWSE) to the supplied repository or a group
   * repository as a member of supplied repository.
   *
   * Examples:
   * Given - repositoryId = raw-hosted
   * Scenario 1. as-repository-view-raw-raw-hosted-read - allowed
   *
   * Scenario 2. as-repository-view-raw-raw-group-read(raw-group contains raw-hosted as a member) - allowed.
   * In this case a user has not got raw-hosted permissions but the user has raw-group permissions.
   * So the user has access to all members of the raw-group.
   *
   */
  Repository getReadableRepository(String repositoryId);

  /**
   * Retrieve all repositories that the user access to.
   */
  List<RepositoryXO> getRepositories();

  /**
   * Retrieves all group repository names that the specified repository is a member of.
   */
  Set<String> findContainingGroups(String repositoryName);

  Optional<Long> getRepositorySize(String repositoryName);
}
