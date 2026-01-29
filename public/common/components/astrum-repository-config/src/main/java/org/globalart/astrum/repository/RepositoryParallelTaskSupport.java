
package org.globalart.astrum.repository;

import java.util.Objects;
import java.util.Set;
import java.util.stream.Stream;
import java.util.stream.StreamSupport;

import javax.annotation.Nonnull;

import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.scheduling.ParallelTaskSupport;

import com.google.common.base.Strings;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.RepositoryTaskSupport.ALL_REPOSITORIES;
import static org.globalart.astrum.repository.RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID;

/**
 * Support for tasks that applies to repositories.
 *
 * If task is configured to run against a repository group (or all repositories) the repository list will be exploded
 * such that the task is run against all repositories referenced in all groups including the group repository itself.
 * Thus, task business logic should not need to process group members themselves
 */
public abstract class RepositoryParallelTaskSupport
    extends ParallelTaskSupport
{
  private RepositoryManager repositoryManager;

  private Type groupType;

  /**
   * @param concurrencyLimit the number of concurrent threads processing the queue allowed.
   */
  protected RepositoryParallelTaskSupport(final int concurrencyLimit) {
    super(concurrencyLimit);
  }

  /**
   * @param taskLoggingEnabled whether task logging should be enabled
   * @param concurrencyLimit the number of concurrent threads processing the queue allowed.
   */
  protected RepositoryParallelTaskSupport(
      final boolean taskLoggingEnabled,
      final int concurrencyLimit)
  {
    super(taskLoggingEnabled, concurrencyLimit);
  }

  @Autowired
  public void install(final RepositoryManager repositoryManager, @Qualifier(GroupType.NAME) final Type groupType) {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.groupType = checkNotNull(groupType, "repository group type required");
  }

  @Override
  protected Stream<Runnable> jobStream(final ProgressLogIntervalHelper progress) {
    return findRepositories()
        .flatMap(repository -> jobStream(progress, repository));
  }

  /**
   * Create a stream of runnables derived from the repository.
   */
  protected abstract Stream<Runnable> jobStream(ProgressLogIntervalHelper progress, final Repository repository);

  /**
   * Return true if the task should be run against specified repository.
   */
  protected abstract boolean appliesTo(final Repository repository);

  /**
   * Extract repository field out of configuration.
   */
  protected String getRepositoryField() {
    return getConfiguration().getString(REPOSITORY_NAME_FIELD_ID);
  }

  /**
   * Returns true if the repository is of type {@link GroupType}
   */
  protected boolean isGroupRepository(final Repository repository) {
    return groupType.equals(repository.getType());
  }

  @Nonnull
  private Stream<Repository> findRepositories() {
    final String repositoryName = getRepositoryField();
    checkArgument(!Strings.isNullOrEmpty(repositoryName));

    Set<String> repositoryNames = Set.of(repositoryName.split(","));

    if (repositoryNames.contains(ALL_REPOSITORIES)) {
      return StreamSupport.stream(repositoryManager.browse().spliterator(), false)
          .filter(this::appliesTo);
    }
    return repositoryNames.stream()
        .map(repositoryManager::get)
        .filter(Objects::nonNull)
        .filter(this::appliesTo);
  }
}
