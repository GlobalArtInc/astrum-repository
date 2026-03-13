
package org.globalart.astrum.repository.search.sql.query.security;

import java.util.Optional;
import java.util.Set;
import java.util.function.Supplier;
import java.util.stream.Collectors;

import com.google.common.base.Suppliers;

import org.globalart.astrum.repository.rest.sql.SearchField;
import org.globalart.astrum.repository.search.sql.SearchMappingService;
import org.globalart.astrum.repository.search.sql.query.DatabaseTypeDetector;
import org.globalart.astrum.repository.search.sql.query.syntax.ExactTerm;
import org.globalart.astrum.repository.search.sql.query.syntax.Expression;
import org.globalart.astrum.repository.search.sql.query.syntax.Operand;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlClause;
import org.globalart.astrum.repository.search.sql.query.syntax.SqlPredicate;
import org.globalart.astrum.repository.search.sql.query.syntax.Term;
import org.globalart.astrum.repository.search.sql.query.syntax.TermCollection;
import org.globalart.astrum.selector.CselSelector;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorEvaluationException;
import org.globalart.astrum.selector.SelectorManager;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Creates SQL filters from content selectors. The expectation is that the generated SQL filters will be used in a SQL
 * WHERE clause.
 */
@Component
public class SearchContentSelectorSqlFilterGenerator
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String PATH = "path";

  private final SelectorManager selectorManager;

  private final SearchMappingService mappingService;

  private final Supplier<CselToExpression> cselToExpressionSupplier;

  @Autowired
  public SearchContentSelectorSqlFilterGenerator(
      final SelectorManager selectorManager,
      final SearchMappingService mappingService,
      final DatabaseTypeDetector databaseTypeDetector)
  {
    checkNotNull(databaseTypeDetector);
    this.selectorManager = checkNotNull(selectorManager);
    this.mappingService = checkNotNull(mappingService);
    this.cselToExpressionSupplier = Suppliers.memoize(() -> databaseTypeDetector.isH2()
        ? new H2CselToExpression()
        : new PostgresCselToExpression());
  }

  /**
   * Maybe create a {@link Expression} for a Content Selector and the repositories to which it applies.
   *
   * @param selector the content selector
   * @param repositories the repository names to which this content selector applies
   * @return
   */
  public Optional<Expression> createFilter(
      final SelectorConfiguration selector,
      final Set<String> repositories)
  {
    if (!CselSelector.TYPE.equals(selector.getType())) {
      log.debug("Content selector is not CSEL: {}", selector.getName());
      return Optional.empty();
    }
    if (repositories.isEmpty()) {
      log.debug("Provided with empty set of repositores for: {}", selector.getName());
      return Optional.empty();
    }

    try {
      Expression filters = transformSelectorToSql(selector);
      return Optional.ofNullable(collectGeneratedSql(filters, repositories));
    }
    catch (SelectorEvaluationException e) {
      log.warn("Problem evaluating selector {} as SQL", selector.getName(), e);
      return Optional.empty();
    }
  }

  private Expression transformSelectorToSql(final SelectorConfiguration selector) throws SelectorEvaluationException {
    SelectorExpressionBuilder selectorBuilder = new SelectorExpressionBuilder(mappingService);
    selectorBuilder.propertyAlias(PATH, SearchField.PATHS);
    selectorManager.toSql(selector, selectorBuilder, cselToExpressionSupplier.get());

    return selectorBuilder.build();
  }

  private Expression collectGeneratedSql(
      final Expression filterExpression,
      final Set<String> repositories)
  {
    return SqlClause.create(Operand.AND, filterExpression, repositoryNameCondition(repositories));
  }

  private SqlPredicate repositoryNameCondition(
      final Set<String> repositories)
  {
    Term terms = TermCollection.create(repositories.stream().map(ExactTerm::new).collect(Collectors.toList()));

    return new SqlPredicate(Operand.IN, SearchField.REPOSITORY_NAME, terms);
  }
}
