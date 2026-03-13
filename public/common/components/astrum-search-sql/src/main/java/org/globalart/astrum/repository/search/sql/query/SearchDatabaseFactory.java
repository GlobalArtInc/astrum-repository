
package org.globalart.astrum.repository.search.sql.query;

import java.util.Optional;

import org.globalart.astrum.repository.rest.sql.SearchField;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Factory that provides the appropriate {@link SearchDatabase} implementation based on the detected database type.
 * This delegates to either H2SearchDB or PostgresSearchDB at runtime after database configuration is loaded.
 */
@Component
@Primary
public class SearchDatabaseFactory
    implements SearchDatabase
{
  private final DatabaseTypeDetector databaseTypeDetector;

  private final ApplicationContext context;

  private SearchDatabase delegate;

  @Autowired
  public SearchDatabaseFactory(
      final DatabaseTypeDetector databaseTypeDetector,
      final ApplicationContext context)
  {
    this.databaseTypeDetector = checkNotNull(databaseTypeDetector);
    this.context = checkNotNull(context);
  }

  /**
   * Returns the appropriate SearchDatabase implementation based on detected database type.
   */
  private SearchDatabase getDelegate() {
    if (delegate == null) {
      delegate = BeanFactoryAnnotationUtils.qualifiedBeanOfType(context, SearchDatabase.class,
          databaseTypeDetector.isH2() ? "h2" : "postgres");
    }
    return delegate;
  }

  @Override
  public Optional<String> getSortColumn(final SearchField field) {
    return getDelegate().getSortColumn(field);
  }

  @Override
  public String formatJsonPath(final String columnName, final String jsonPath) {
    return getDelegate().formatJsonPath(columnName, jsonPath);
  }
}
