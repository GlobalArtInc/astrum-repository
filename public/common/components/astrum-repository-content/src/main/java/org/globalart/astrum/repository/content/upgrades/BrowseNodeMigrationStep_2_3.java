
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.ResultSet;
import java.sql.SQLException;
import java.util.Optional;
import java.util.StringJoiner;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.repository.RepositoryTaskSupport;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodesTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class BrowseNodeMigrationStep_2_3
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final TaskScheduler taskScheduler;

  private final UpgradeTaskScheduler upgradeTaskScheduler;

  private static final String CONTENT_REPOSITORY_TABLE = "conan_content_repository";

  private static final String SELECT_REPOSITORY_NAMES = "SELECT R.name " +
      "FROM repository R, " + CONTENT_REPOSITORY_TABLE + " C " +
      "WHERE R.id = C.config_repository_id";

  @Autowired
  public BrowseNodeMigrationStep_2_3(
      final TaskScheduler taskScheduler,
      final UpgradeTaskScheduler upgradeTaskScheduler)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.upgradeTaskScheduler = checkNotNull(upgradeTaskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.3");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (!tableExists(connection, CONTENT_REPOSITORY_TABLE)) {
      log.debug("No Conan table present to rebuild browse nodes");
      return;
    }

    String repositoryNames = getRepositoryNames(connection);

    if (!repositoryNames.isEmpty()) {
      scheduleRebuildBrowseNodesTask(repositoryNames);
    }
    else {
      log.debug("No Conan repositories found to rebuild browse nodes");
    }
  }

  protected String getRepositoryNames(final Connection connection) throws IllegalStateException {
    StringJoiner repositoryNames = new StringJoiner(",");
    try (PreparedStatement statement = connection.prepareStatement(SELECT_REPOSITORY_NAMES)) {
      try (ResultSet resultSet = statement.executeQuery()) {
        while (resultSet.next()) {
          repositoryNames.add(resultSet.getString(1));
        }
      }
    }
    catch (SQLException e) {
      throw new IllegalStateException("Failed to get repository names", e);
    }
    return repositoryNames.toString();
  }

  private void scheduleRebuildBrowseNodesTask(final String repositories) {
    TaskConfiguration configuration =
        taskScheduler.createTaskConfigurationInstance(RebuildBrowseNodesTaskDescriptor.TYPE_ID);
    configuration.setName("Rebuild browse nodes for Conan repositories");
    configuration.setString(RepositoryTaskSupport.REPOSITORY_NAME_FIELD_ID, repositories);
    upgradeTaskScheduler.schedule(configuration);
    log.info("Scheduled post-startup task to rebuild browse nodes for Conan repositories: {}", repositories);
  }
}
