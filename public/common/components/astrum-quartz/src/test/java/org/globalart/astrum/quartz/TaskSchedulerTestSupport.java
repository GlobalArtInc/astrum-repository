
package org.globalart.astrum.quartz;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.Date;
import java.util.UUID;

import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.app.ApplicationVersion;
import org.globalart.astrum.common.app.BaseUrlManager;
import org.globalart.astrum.common.cooperation2.Cooperation2Selector;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.log.LastShutdownTimeService;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.quartz.TaskSchedulerTestSupport.TaskSchedulerTestConfiguration;
import org.globalart.astrum.quartz.internal.QuartzSchedulerProvider;
import org.globalart.astrum.quartz.internal.QuartzSchedulerSPI;
import org.globalart.astrum.quartz.internal.datastore.QuartzDAO;
import org.globalart.astrum.quartz.internal.datastore.QuartzJobDataTypeHandler;
import org.globalart.astrum.quartz.internal.store.ConfigStoreConnectionProvider;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.scheduling.schedule.Hourly;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.spi.SchedulerSPI;
import org.globalart.astrum.testcommon.event.SimpleEventManager;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseExtension;
import org.globalart.astrum.testdb.TestDataSessionSupplier;
import org.globalart.astrum.thread.DatabaseStatusDelayedExecutor;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Answers;
import org.quartz.Scheduler;
import org.quartz.utils.ConnectionProvider;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.ComponentScan.Filter;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.FilterType;
import org.springframework.context.annotation.FullyQualifiedAnnotationBeanNameGenerator;
import org.springframework.context.annotation.Primary;
import org.springframework.test.context.TestPropertySource;

import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static org.awaitility.Awaitility.await;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@SpringBootTest(classes = TaskSchedulerTestConfiguration.class)
@ExtendWith(DatabaseExtension.class)
@TestPropertySource(properties = {
    "astrum.quartz.poolSize=8",
    "astrum.quartz.jobstore.jdbc=true"
})
public abstract class TaskSchedulerTestSupport
{
  @Autowired
  private SchedulerSPI scheduler;

  @Autowired
  protected TaskScheduler taskScheduler;

  @Autowired
  private QuartzSchedulerProvider schedulerProvider;

  @Autowired
  protected EventManager eventManager;

  @DataSessionConfiguration(daos = QuartzDAO.class, typeHandlers = QuartzJobDataTypeHandler.class)
  protected TestDataSessionSupplier dataSessionSupplier;

  // max time to wait for task completion
  public static final int RUN_TIMEOUT = 2000;

  public static final String RESULT = "This is the expected result";

  @BeforeEach
  void start() throws Exception {
    // Set the connection provider to use the TestDataSessionSupplier from DatabaseExtension
    TestConfigStoreConnectionProvider connectionProvider = TaskSchedulerTestConfiguration.getTestConnectionProvider();
    if (connectionProvider != null && dataSessionSupplier != null) {
      connectionProvider.setDataSessionSupplier(dataSessionSupplier);
    }

    schedulerProvider.start();
    scheduler.start();
    scheduler.resume();
  }

  @AfterEach
  void after() throws Exception {
    scheduler.pause();
    scheduler.stop();
    schedulerProvider.stop();
  }

  protected Scheduler getScheduler() {
    return ((QuartzSchedulerSPI) scheduler).getScheduler();
  }

  /**
   * Test ConfigStoreConnectionProvider that can switch between mock and real DataSessionSupplier
   */
  public static class TestConfigStoreConnectionProvider
      extends ConfigStoreConnectionProvider
  {
    private TestDataSessionSupplier testDataSessionSupplier;

    public TestConfigStoreConnectionProvider(DataSessionSupplier mockDataSessionSupplier) {
      super(mockDataSessionSupplier);
    }

    public void setDataSessionSupplier(TestDataSessionSupplier dataSessionSupplier) {
      this.testDataSessionSupplier = dataSessionSupplier;
    }

    @Override
    public Connection getConnection() throws SQLException {
      if (testDataSessionSupplier != null) {
        return testDataSessionSupplier.openConnection(DEFAULT_DATASTORE_NAME);
      }
      return super.getConnection();
    }
  }

  protected void assertRunningTaskCount(int expectedCount) {
    // pool maintenance might not be done when a task's future returns so polling is in order to be safe
    await().atMost(RUN_TIMEOUT, MILLISECONDS).until(() -> taskScheduler.getRunningTaskCount(), is(expectedCount));
  }

  protected void assertExecutedTaskCount(int expectedCount) {
    await().atMost(RUN_TIMEOUT, MILLISECONDS).until(() -> taskScheduler.getExecutedTaskCount(), is(expectedCount));
  }

  protected void assertTaskState(final TaskInfo taskInfo, final TaskState expectedState) {
    assertTaskState(RUN_TIMEOUT, taskInfo, expectedState);
  }

  protected void assertTaskState(final long waitMillis, final TaskInfo taskInfo, final TaskState expectedState) {
    // unfortunately, a task's Future.get() returns before the task state is updated so polling is in order to be safe
    await().atMost(waitMillis, MILLISECONDS).until(() -> taskInfo.getCurrentState().getState(), is(expectedState));
  }

  /**
   * Creates a {@link Hourly} schedule that is about to start in .5sec in future, as we now "step" triggers if
   * in past, instead executing them immediately.
   */
  protected Schedule hourly() {
    return taskScheduler.getScheduleFactory().hourly(new Date(System.currentTimeMillis() + 1000L));
  }

  protected TaskInfo createTask(String typeId) {
    return createTask(typeId, hourly());
  }

  protected TaskInfo createTask(String typeId, Schedule schedule) {
    final TaskConfiguration taskConfiguration = taskScheduler
        .createTaskConfigurationInstance(typeId);
    taskConfiguration.setString(SleeperTask.RESULT_KEY, RESULT);
    return taskScheduler.scheduleTask(taskConfiguration, schedule);
  }

  @Configuration
  @ComponentScan(
      basePackages = {
          "org.globalart.astrum.scheduling", "org.globalart.astrum.quartz"
      },
      excludeFilters = @Filter(type = FilterType.REGEX,
          pattern = {
              ".*H2VersionUpgrader", ".*MyBatisCipher", ".*SecretTypeHandler"
          }),
      nameGenerator = FullyQualifiedAnnotationBeanNameGenerator.class)
  public static class TaskSchedulerTestConfiguration
  {
    private static TestConfigStoreConnectionProvider testConnectionProvider;

    @Bean
    public DataSessionSupplier dataSessionSupplier() {
      return mock(DataSessionSupplier.class);
    }

    @Bean
    @Primary
    public ConnectionProvider connectionProvider(DataSessionSupplier mockSupplier) {
      testConnectionProvider = new TestConfigStoreConnectionProvider(mockSupplier);
      return testConnectionProvider;
    }

    public static TestConfigStoreConnectionProvider getTestConnectionProvider() {
      return testConnectionProvider;
    }

    @Bean
    public Cooperation2Selector cooperation2Selector() {
      return mock(Cooperation2Selector.class, Answers.RETURNS_MOCKS);
    }

    @Bean
    public EventManager eventManager() {
      return new SimpleEventManager();
    }

    @Bean
    public ApplicationDirectories applicationDirectories() {
      return mock(ApplicationDirectories.class);
    }

    @Bean
    public BaseUrlManager baseUrlManager() {
      return mock(BaseUrlManager.class);
    }

    @Bean
    public NodeAccess nodeAccess() {
      NodeAccess nodeAccess = mock(NodeAccess.class);
      when(nodeAccess.getId()).thenReturn(UUID.randomUUID().toString());
      return nodeAccess;
    }

    @Bean
    public LastShutdownTimeService lastShutdownTimeService() {
      return mock(LastShutdownTimeService.class);
    }

    @Bean
    public DatabaseStatusDelayedExecutor databaseStatusDelayedExecutor() {
      return mock(DatabaseStatusDelayedExecutor.class);
    }

    @Bean
    public DatabaseCheck databaseCheck() {
      DatabaseCheck databaseCheck = mock(DatabaseCheck.class);
      when(databaseCheck.isAllowedByVersion(any())).thenReturn(true);
      return databaseCheck;
    }

    @Bean
    public ApplicationVersion applicationVersion() {
      ApplicationVersion applicationVersion = mock(ApplicationVersion.class);
      when(applicationVersion.getVersion()).thenReturn("test-version-1");
      when(applicationVersion.getBuildRevision()).thenReturn("test-revision-1");
      return applicationVersion;
    }

  }
}
