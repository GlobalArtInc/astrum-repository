
package org.globalart.astrum.repository;

import java.util.Set;

import javax.annotation.Nonnull;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.failure.MultipleFailures;
import org.globalart.astrum.common.stateguard.InvalidStateException;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.scheduling.TaskInterruptedException;
import org.globalart.astrum.scheduling.TaskSupport;

import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Iterables;
import com.google.common.collect.Sets;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;
import static java.util.Arrays.asList;
import static java.util.Arrays.stream;
import static java.util.stream.Collectors.toList;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Support for tasks that applies to repositories.
 *
 * If task is configured to run against a repository group (or all repositories) the repository list will be exploded
 * such that the task is run against all repositories referenced in all groups including the group repository itself.
 * Thus, task business logic should not need to process group members themselves
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public abstract class RepositoryTaskSupport
    extends TaskSupport
{
  public static final String REPOSITORY_NAME_FIELD_ID = "repositoryName";

  public static final String ALL_REPOSITORIES = "*";

  private RepositoryManager repositoryManager;

  private Type groupType;

  private Set<Repository> processedRepositories;

  protected RepositoryTaskSupport() {
    super();
  }

  protected RepositoryTaskSupport(final boolean taskLoggingEnabled) {
    super(taskLoggingEnabled);
  }

  @Autowired
  public void install(final RepositoryManager repositoryManager, @Qualifier(GroupType.NAME) final Type groupType) {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.groupType = checkNotNull(groupType, "repository group type required");
  }

  @Override
  protected Object execute() throws Exception {
    processedRepositories = Sets.newHashSet();
    MultipleFailures failures = new MultipleFailures();
    for (Repository repository : findRepositories()) {
      if (isCanceled()) {
        break;
      }

      try {
        execute(repository);
      }
      catch (TaskInterruptedException e) { // NOSONAR
        throw e;
      }
      catch (MissingFacetException | InvalidStateException e) {
        // Repository was deleted concurrently mid-iteration (ASTRUM-53351); skip it rather than
        // escalating to task failure. See ASTRUM-50820 for the equivalent UI-path handling.
        // Note the log-level divergence from ASTRUM-50820: that UI-status path logs at DEBUG (low-signal
        // per-request status queries), whereas these background task sweeps log at WARN because a skipped
        // repository in a full-iteration task is worth surfacing. Do not "unify" the two levels.
        // Trade-off: InvalidStateException is caught broadly here. It normally signals a @Guarded
        // method invoked on a repository that was just stopped/deleted, but it can also fire for a
        // non-deletion cause (e.g. a component that has not yet started, or a misconfiguration),
        // which will now surface only as a WARN rather than a task failure. This is an accepted
        // trade-off (per ASTRUM-53351) to avoid noisy ERROR stack traces from the common
        // concurrent-deletion race that would otherwise drown out genuine failures.
        log.warn("Skipping repository '{}' for task '{}' as it appears to have been removed during execution: {}",
            repository.getName(), getMessage(), e.getMessage());
      }
      catch (Exception e) {
        log.error("Failed to run task '{}' on repository '{}'", getMessage(), repository.getName(), e);
        failures.add(e);
      }
    }

    failures.maybePropagate(String.format("Failed to run task '%s'", getMessage()));
    return null;
  }

  @Nonnull
  private Iterable<Repository> findRepositories() {
    final String repositoryName = getRepositoryField();
    checkArgument(!Strings.isNullOrEmpty(repositoryName));
    if (ALL_REPOSITORIES.equals(repositoryName)) {
      return Iterables.filter(repositoryManager.browse(), this::appliesTo);
    }
    else if (repositoryName.contains(",")) {
      String[] repositoryNames = repositoryName.split(",");

      if (asList(repositoryNames).contains(ALL_REPOSITORIES)) {
        return Iterables.filter(repositoryManager.browse(), this::appliesTo);
      }
      else {
        return stream(repositoryNames).map(repositoryManager::get).filter(this::appliesTo).collect(toList());
      }
    }
    else {
      Repository repository = repositoryManager.get(repositoryName);
      if (repository == null) {
        log.warn("Repository '{}' was not found while running task '{}'", repositoryName, getMessage());
        throw new TaskInterruptedException(String.format("Repository '%s' was not found.", repositoryName), true);
      }
      checkState(appliesTo(repository));
      return ImmutableList.of(repository);
    }
  }

  /**
   * Extract repository field out of configuration.
   */
  protected String getRepositoryField() {
    return getConfiguration().getString(REPOSITORY_NAME_FIELD_ID);
  }

  /**
   * Get the repository manager.
   */
  protected RepositoryManager getRepositoryManager() {
    return repositoryManager;
  }

  /**
   * Returns true if the repository is of type {@link GroupType}
   *
   * @since 3.6.1
   */
  protected boolean isGroupRepository(final Repository repository) {
    return groupType.equals(repository.getType());
  }

  /**
   * Tracks processed repositories
   *
   * @since 3.6.1
   */
  protected void markProcessed(final Repository repository) {
    processedRepositories.add(repository);
  }

  /**
   * Returns true if the specified repository has already been processed during task execution
   *
   * @since 3.6.1
   */
  protected boolean hasBeenProcessed(final Repository repository) {
    return processedRepositories.contains(repository);
  }

  /**
   * Execute against specified repository.
   */
  protected abstract void execute(final Repository repository);

  /**
   * Return true if the task should be run against specified repository.
   */
  protected abstract boolean appliesTo(final Repository repository);
}
