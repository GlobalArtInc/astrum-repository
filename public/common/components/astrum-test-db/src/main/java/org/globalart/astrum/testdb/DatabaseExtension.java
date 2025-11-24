
package org.globalart.astrum.testdb;

import java.lang.reflect.Field;
import java.sql.Connection;
import java.sql.DriverManager;
import java.sql.PreparedStatement;
import java.sql.SQLException;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.stream.Stream;

import javax.annotation.Nullable;

import org.junit.jupiter.api.extension.AfterAllCallback;
import org.junit.jupiter.api.extension.BeforeAllCallback;
import org.junit.jupiter.api.extension.Extension;
import org.junit.jupiter.api.extension.ExtensionContext;
import org.junit.jupiter.api.extension.TestTemplateInvocationContext;
import org.junit.jupiter.api.extension.TestTemplateInvocationContextProvider;
import org.junit.platform.commons.support.AnnotationSupport;
import org.testcontainers.containers.PostgreSQLContainer;
import org.testcontainers.containers.output.Slf4jLogConsumer;
import org.testcontainers.utility.DockerImageName;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.slf4j.LoggerFactory.getLogger;

/**
 * <p>
 * JUnit 5 extension to inject an instance of {@code DataSessionSupplier} into tests.
 * </p>
 *
 * <p>
 * Annotate the test class with {@code @ExtendWith(DatabaseExtension.class)}
 * </p>
 *
 * <p>
 * Create a field and annotate it, e.g.
 * </p>
 *
 * <pre>
 * &#64;DataSessionConfiguration(MyDAO.class)
 * TestDataSessionSupplier dataSessionSupplier;
 *
 * &#64;DatabaseTest
 * void testMyDAO() {
 * }
 * </pre>
 */
public class DatabaseExtension
    implements Extension, BeforeAllCallback, AfterAllCallback, TestTemplateInvocationContextProvider
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String JDBC_URL = System.getProperty("test.jdbcUrl");

  private static final boolean EXTERNAL_POSTGRES = JDBC_URL != null && JDBC_URL.startsWith("jdbc:postgresql:");

  private PostgreSQLContainer<?> postgres;

  @Override
  public void beforeAll(final ExtensionContext context) throws Exception {
    if (findDataSessionConfiguration(context).postgresql() && !EXTERNAL_POSTGRES) {
      startPostgres();
    }
  }

  @Override
  public void afterAll(final ExtensionContext context) throws Exception {
    stopPostgres();
  }

  @Override
  public boolean supportsTestTemplate(final ExtensionContext context) {
    return context.getTestMethod().isPresent();
  }

  @Override
  public boolean mayReturnZeroTestTemplateInvocationContexts(final ExtensionContext context) {
    return true;
  }

  @Override
  public Stream<TestTemplateInvocationContext> provideTestTemplateInvocationContexts(
      final ExtensionContext extensionContext)
  {
    DataSessionConfiguration session = findDataSessionConfiguration(extensionContext);
    DatabaseTest databaseTest = findDatabaseTest(extensionContext);
    String displayName = extensionContext.getDisplayName();
    Map<String, String> databaseToUrl = new HashMap<>();

    if (session.h2() && databaseTest.h2()) {
      databaseToUrl.put("[h2] " + displayName, "jdbc:h2:mem:${storeName}");
    }
    if ((session.postgresql() || EXTERNAL_POSTGRES) && databaseTest.postgresql()) {
      String schemaName = schemaName(extensionContext, displayName);
      createSchema(schemaName);
      databaseToUrl.put("[postgres] " + displayName, getPostgresqlUrl(schemaName));
    }

    return databaseToUrl.entrySet()
        .stream()
        .map(entry -> new DatabaseTestTemplateInvocationContext(entry.getKey(), entry.getValue()));
  }

  static DataSessionConfiguration findDataSessionConfiguration(final ExtensionContext context) {
    Class<?> testClass = context.getRequiredTestClass();
    while (testClass != null && testClass != Object.class) {
      for (Field field : testClass.getDeclaredFields()) {
        if (field.isAnnotationPresent(DataSessionConfiguration.class)) {
          return field.getAnnotation(DataSessionConfiguration.class);
        }
      }
      testClass = testClass.getSuperclass();
    }
    throw new IllegalStateException("No @DataSessionConfiguration found");
  }

  static DatabaseTest findDatabaseTest(final ExtensionContext context) {
    return AnnotationSupport.findAnnotation(context.getRequiredTestMethod(), DatabaseTest.class)
        .orElseThrow();
  }

  private void startPostgres() {
    // Astrum support policy is we support versions of PostgreSQL which are currently supported
    // https://www.postgresql.org/support/versioning/
    // 14 is the minimum support version as of November 2025
    // switch to alpine images due to policy violations
    postgres = new PostgreSQLContainer<>(
        DockerImageName.parse("globalart.repo.globalart.app/docker-all/postgres:15.15-alpine")
            .asCompatibleSubstituteFor("postgres"));

    postgres.withLogConsumer(new Slf4jLogConsumer(getLogger("postgres")))
        .withCommand("postgres", "-c",
            "max_connections=110");
    postgres.start();

    postgres.withUrlParam("user", "test").withUrlParam("password", "test");

    log.info("Started postgres {}", postgres.getJdbcUrl());
  }

  private void stopPostgres() {
    if (postgres != null) {
      postgres.close();
      postgres = null;
    }
  }

  private void createSchema(@Nullable final String name) {
    if (name == null) {
      return;
    }

    try (Connection conn = DriverManager.getConnection(getPostgresqlUrl(null))) {
      if (EXTERNAL_POSTGRES) {
        try (PreparedStatement statement = conn.prepareStatement("DROP SCHEMA IF EXISTS " + name + " CASCADE")) {
          statement.execute();
        }
      }
      try (PreparedStatement statement = conn.prepareStatement("CREATE SCHEMA " + name)) {
        statement.execute();
      }
    }
    catch (SQLException e) {
      throw new RuntimeException(e);
    }
  }

  private String getPostgresqlUrl(@Nullable final String schema) {
    String baseUrl = EXTERNAL_POSTGRES ? JDBC_URL : postgres.getJdbcUrl();
    if (schema == null) {
      return baseUrl;
    }
    return baseUrl + (baseUrl.contains("?") ? "&" : "?") + "currentSchema=" + schema;
  }

  private static String schemaName(final ExtensionContext context, final String displayName) {
    String name = displayName.replaceAll("[^a-zA-Z0-9_]", "");
    if (!EXTERNAL_POSTGRES) {
      return name;
    }
    String className = context.getRequiredTestClass().getSimpleName().replaceAll("[^a-zA-Z0-9_]", "");
    return (className + "_" + name).toLowerCase(Locale.ROOT);
  }

  private static class DatabaseTestTemplateInvocationContext
      implements TestTemplateInvocationContext
  {
    private final String testName;

    private final String jdbcUrl;

    DatabaseTestTemplateInvocationContext(final String testName, final String jdbcUrl) {
      this.testName = testName;
      this.jdbcUrl = jdbcUrl;
    }

    @Override
    public String getDisplayName(final int invocationIndex) {
      return testName;
    }

    @Override
    public List<Extension> getAdditionalExtensions() {
      return List.of(new DatabaseSpecificExtension(jdbcUrl));
    }
  }
}
