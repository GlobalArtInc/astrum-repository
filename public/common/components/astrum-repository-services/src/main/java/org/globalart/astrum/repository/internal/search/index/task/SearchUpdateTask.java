
package org.globalart.astrum.repository.internal.search.index.task;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.index.RebuildIndexTaskDescriptor;
import org.globalart.astrum.repository.search.index.SearchIndexFacet;
import org.globalart.astrum.repository.search.index.SearchUpdateService;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskSupport;

import com.google.common.collect.ImmutableMap;

import static java.util.Objects.requireNonNull;
import static org.globalart.astrum.repository.RepositoryTaskSupport.ALL_REPOSITORIES;
import static org.globalart.astrum.repository.internal.search.index.task.SearchUpdateTaskDescriptor.REPOSITORY_NAMES_FIELD_ID;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Task that updates repository search indexes that are out of date.
 *
 * @since 3.37
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SearchUpdateTask
    extends TaskSupport
    implements Cancelable
{
  private final RepositoryManager repositoryManager;

  private final SearchUpdateService searchUpdateService;

  private final TaskScheduler taskScheduler;

  @Autowired
  public SearchUpdateTask(
      final RepositoryManager repositoryManager,
      final SearchUpdateService searchUpdateService,
      final TaskScheduler taskScheduler)
  {
    this.repositoryManager = requireNonNull(repositoryManager);
    this.searchUpdateService = requireNonNull(searchUpdateService);
    this.taskScheduler = taskScheduler;
  }

  @Override
  protected Object execute() {
    if (taskScheduler.findWaitingTask(RebuildIndexTaskDescriptor.TYPE_ID,
        ImmutableMap.of(RebuildIndexTaskDescriptor.REPOSITORY_NAME_FIELD_ID, ALL_REPOSITORIES))) {
      log.info("A task of type {} is already scheduled for all repositories, cancelling task {} ",
          RebuildIndexTaskDescriptor.TYPE_ID, SearchUpdateTaskDescriptor.TYPE_ID);
      taskScheduler.cancel(SearchUpdateTaskDescriptor.TYPE_ID, true);

      return null;
    }

    String[] repositoryNames = getRepositoryNamesField();

    for (String name : repositoryNames) {
      Repository repository = repositoryManager.get(name);
      if (repository != null) {
        try {
          log.info("Updating search index for repo {}", name);
          SearchIndexFacet searchIndexFacet = repository.facet(SearchIndexFacet.class);
          searchIndexFacet.rebuildIndex();
          searchUpdateService.doneReindexing(repository);
          log.info("Completed update of search index for repo {}", name);
        }
        catch (Exception e) {
          log.error("Could not perform search index update for repo {}, {}", name, e.getMessage());
        }
      }
    }

    return null;
  }

  @Override
  public String getMessage() {
    return "Updating search indexes of " + String.join(",", getRepositoryNamesField());
  }

  /**
   * Extract repository field out of configuration.
   */
  private String[] getRepositoryNamesField() {
    String value = getConfiguration().getString(REPOSITORY_NAMES_FIELD_ID);
    if (value != null) {
      return value.split(",");
    }
    return new String[]{};
  }
}
