package org.globalart.astrum.repository.search.sql.store;

import java.sql.Connection;
import java.sql.SQLException;

import org.globalart.astrum.repository.search.sql.query.SearchConditionFactory;
import org.globalart.astrum.repository.search.sql.query.h2.H2SearchConditionFactory;
import org.globalart.astrum.repository.search.sql.query.h2.H2SearchDB;
import org.globalart.astrum.repository.search.sql.query.postgres.PostgresSearchConditionFactory;
import org.globalart.astrum.repository.search.sql.query.postgres.PostgresSearchDB;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

final class SearchConditionFactories
{
  private SearchConditionFactories() {
  }

  static SearchConditionFactory forSession(final TestDataSessionSupplier sessionSupplier) {
    try (Connection connection = sessionSupplier.openConnection()) {
      String product = connection.getMetaData().getDatabaseProductName();
      if ("PostgreSQL".equalsIgnoreCase(product)) {
        return new PostgresSearchConditionFactory(new PostgresSearchDB());
      }
      return new H2SearchConditionFactory(new H2SearchDB());
    }
    catch (SQLException e) {
      throw new IllegalStateException(e);
    }
  }
}
