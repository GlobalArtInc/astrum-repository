
package org.globalart.astrum.repository.search.sql.query;

import org.globalart.astrum.repository.search.sql.ExpressionGroup;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.BeanFactoryAnnotationUtils;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Factory that provides the appropriate {@link SearchConditionFactory} implementation based on the detected database
 * type.
 * This delegates to either H2SearchConditionFactory or PostgresSearchConditionFactory at runtime after database
 * configuration is loaded.
 */
@Component
@Primary
public class SearchConditionFactoryImpl
    implements SearchConditionFactory
{
  private final DatabaseTypeDetector databaseTypeDetector;

  private final ApplicationContext context;

  private SearchConditionFactory delegate;

  @Autowired
  public SearchConditionFactoryImpl(
      final DatabaseTypeDetector databaseTypeDetector,
      final ApplicationContext context)
  {
    this.databaseTypeDetector = checkNotNull(databaseTypeDetector);
    this.context = checkNotNull(context);
  }

  /**
   * Returns the appropriate SearchConditionFactory implementation based on detected database type.
   */
  private SearchConditionFactory getDelegate() {
    if (delegate == null) {
      delegate = BeanFactoryAnnotationUtils.qualifiedBeanOfType(context, SearchConditionFactory.class,
          databaseTypeDetector.isH2() ? "h2" : "postgres");
    }
    return delegate;
  }

  @Override
  public SqlSearchQueryConditionGroup build(final ExpressionGroup expressionGroup) {
    return getDelegate().build(expressionGroup);
  }
}
