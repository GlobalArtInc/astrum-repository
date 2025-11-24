
package org.globalart.astrum.testdb;

import java.sql.Connection;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.globalart.astrum.datastore.api.DataAccess;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DataStore;
import org.globalart.astrum.datastore.api.DataStoreConfiguration;
import org.globalart.astrum.datastore.api.DataStoreNotFoundException;
import org.globalart.astrum.datastore.mybatis.MyBatisDataStore;
import org.globalart.astrum.transaction.TransactionIsolation;

import org.apache.ibatis.plugin.Interceptor;
import org.apache.ibatis.type.TypeHandler;
import org.junit.rules.ExternalResource;
import org.junit.runner.Description;
import org.junit.runners.model.Statement;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.testcontainers.containers.PostgreSQLContainer;
import org.testcontainers.containers.output.Slf4jLogConsumer;
import org.testcontainers.utility.DockerImageName;

import static com.google.common.base.Throwables.throwIfUnchecked;
import static com.google.common.collect.ImmutableMap.toImmutableMap;
import static java.util.Arrays.stream;
import static java.util.Optional.ofNullable;
import static java.util.function.Function.identity;
import static java.util.stream.Stream.concat;
import static java.util.stream.Stream.of;
import static org.globalart.astrum.common.property.SystemPropertiesHelper.getBoolean;
import static org.globalart.astrum.common.text.Strings2.isBlank;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.datastore.mybatis.MyBatisDataStoreDescriptor.JDBC_URL;

/**
 * JUnit rule to supply {@link DataSession}s without needing the full store ceremony.
 *
 * @since 3.19
 */
public class DataSessionRule
    extends ExternalResource
    implements DataSessionSupplier
{
  private static final Logger log = LoggerFactory.getLogger(DataSessionRule.class);

  private final Map<String, String> attributes = new HashMap<>();

  private final List<Class<? extends DataAccess>> configAccessTypes = new ArrayList<>();

  private final List<Class<? extends DataAccess>> contentAccessTypes = new ArrayList<>();

  private final List<TypeHandler<?>> typeHandlers = new ArrayList<>();

  private final List<Interceptor> interceptors = new ArrayList<>();

  private final Map<String, MyBatisDataStore> stores;

  private String jdbcUrl;

  private PostgreSQLContainer<?> postgres;

  /**
   * Supplies in-memory config sessions.
   */
  public DataSessionRule() {
    this(DEFAULT_DATASTORE_NAME);
  }

  /**
   * Supplies in-memory sessions for the named stores.
   */
  public DataSessionRule(final String storeName, final String... storeNames) {
    stores = concat(of(storeName), stream(storeNames)).collect(toImmutableMap(identity(), this::newStore));
    jdbcUrl = discoverJdbcUrl();
  }

  /**
   * Applies the given attribute to all sessions managed by this rule.
   */
  public DataSessionRule attribute(final String key, final String value) {
    attributes.put(key, value);
    return this;
  }

  /**
   * Registers the given DAO with the appropriate sessions managed by this rule.
   */
  public DataSessionRule access(final Class<? extends DataAccess> accessType) {
    if (ContentDataAccess.class.isAssignableFrom(accessType)) {
      contentAccessTypes.add(accessType);
    }
    else {
      configAccessTypes.add(accessType);
    }
    return this;
  }

  /**
   * Registers the given type handler with all sessions managed by this rule.
   */
  public DataSessionRule handle(final TypeHandler<?> typeHandler) {
    typeHandlers.add(typeHandler);
    return this;
  }

  /**
   * Registers the given interceptor with all sessions managed by this rule.
   */
  public DataSessionRule intercept(final Interceptor interceptor) {
    interceptors.add(interceptor);
    return this;
  }

  /**
   * Discover the JDBC URL to run the tests against.
   */
  protected String discoverJdbcUrl() {
    String url = System.getProperty("test.jdbcUrl");
    if (isBlank(url)) {
      if (getBoolean("test.postgres", false)) {
        url = startPostgres();
      }
      else {
        url = "jdbc:h2:mem:${storeName}";
      }
    }
    return url;
  }

  @Override
  protected void before() {
    stores.forEach((storeName, store) -> {
      try {
        store.start();

        typeHandlers.forEach(store::register);
        interceptors.forEach(store::register);

        if (DEFAULT_DATASTORE_NAME.equals(storeName)) {
          configAccessTypes.forEach(store::register);
          contentAccessTypes.forEach(store::register);
        }
      }
      catch (Exception e) {
        log.warn("Problem starting {}", storeName, e);
        throwIfUnchecked(e);
        throw new RuntimeException(e);
      }
    });
  }

  @Override
  public Statement apply(final Statement base, final Description description) {
    return new Statement()
    {
      @Override
      public void evaluate() throws Throwable {
        try {
          attribute(JDBC_URL, jdbcUrl);
          before();
          try {
            base.evaluate();
          }
          finally {
            after();
          }
        }
        finally {
          if (postgres != null) {
            stopPostgres();
          }
        }
      }
    };
  }

  private MyBatisDataStore newStore(final String storeName) {
    DataStoreConfiguration config = new DataStoreConfiguration();
    config.setName(storeName);
    config.setSource("test");
    config.setType("jdbc");
    config.setAttributes(attributes);

    MyBatisDataStore store = new MyBatisDataStore();
    store.setConfiguration(config);
    return store;
  }

  @Override
  public DataSession<?> openSession(final String storeName) {
    return ofNullable(stores.get(storeName)).orElseThrow(() -> new DataStoreNotFoundException(storeName)).openSession();
  }

  @Override
  public DataSession<?> openSerializableTransactionSession(final String storeName) {
    return ofNullable(stores.get(storeName)).orElseThrow(() -> new DataStoreNotFoundException(storeName))
        .openSession(TransactionIsolation.SERIALIZABLE);
  }

  @Override
  public Connection openConnection(final String storeName) throws SQLException {
    return ofNullable(stores.get(storeName)).orElseThrow(() -> new DataStoreNotFoundException(storeName))
        .openConnection();
  }

  public Optional<DataStore<?>> getDataStore(final String storeName) {
    return Optional.ofNullable(stores.get(storeName));
  }

  @Override
  protected void after() {
    stores.values().forEach(t -> {
      try {
        t.stop();
      }
      catch (Exception e) {
        log.warn("Problem stopping {}", t, e);
      }
    });
  }

  protected String startPostgres() {
    // 13 is the minimum support version

    postgres = new PostgreSQLContainer<>(DockerImageName.parse("globalart.repo.globalart.app/docker-all/postgres:13.20")
        .asCompatibleSubstituteFor("postgres"));

    postgres.withLogConsumer(new Slf4jLogConsumer(LoggerFactory.getLogger("postgres")))
        .withCommand("postgres", "-c", "max_connections=110");
    postgres.start();

    // use the same underlying PostgreSQL database as backing for each store
    return postgres
        .withUrlParam("user", "test")
        .withUrlParam("password", "test")
        .getJdbcUrl();
  }

  protected void stopPostgres() {
    postgres.close();
    postgres = null;
  }
}
