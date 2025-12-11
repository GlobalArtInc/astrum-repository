
package org.globalart.astrum.internal.selector;

import java.util.List;
import java.util.Map;
import java.util.stream.Collectors;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.selector.JexlSelector;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorEvaluationException;
import org.globalart.astrum.selector.SelectorFilterBuilder;
import org.globalart.astrum.selector.SelectorManager;
import org.globalart.astrum.selector.SelectorSqlBuilder;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.util.Objects.requireNonNull;
import org.springframework.stereotype.Component;

@Component
public class SelectorFilterBuilderImpl
    implements SelectorFilterBuilder
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static String FILTER_PARAMS = "filterParams";

  private final SelectorManager selectorManager;

  @Autowired
  public SelectorFilterBuilderImpl(final SelectorManager selectorManager) {
    this.selectorManager = requireNonNull(selectorManager);
  }

  @Nullable
  public String buildFilter(
      final String format,
      final String pathAlias,
      final List<SelectorConfiguration> selectors,
      final Map<String, Object> filterParameters)
  {
    List<SelectorConfiguration> activeSelectors =
        selectors.stream().filter(s -> !JexlSelector.TYPE.equals(s.getType())).collect(Collectors.toList());

    if (activeSelectors.isEmpty()) {
      return null;
    }

    StringBuilder filterBuilder = new StringBuilder();

    if (activeSelectors.size() > 1) {
      filterBuilder.append('(');
    }

    SelectorSqlBuilder sqlBuilder = new SelectorSqlBuilder()
        .propertyAlias("path", pathAlias)
        .propertyAlias("format", "'" + format + "'")
        .parameterPrefix("#{" + FILTER_PARAMS + ".")
        .parameterSuffix("}");

    appendSelectors(filterBuilder, sqlBuilder, activeSelectors, filterParameters);

    if (activeSelectors.size() > 1) {
      filterBuilder.append(')');
    }

    return filterBuilder.toString();
  }

  public void appendSelectors(
      final StringBuilder filterBuilder,
      final SelectorSqlBuilder sqlBuilder,
      final List<SelectorConfiguration> selectors,
      final Map<String, Object> filterParameters)
  {
    int selectorCount = 0;

    for (SelectorConfiguration selector : selectors) {
      try {
        sqlBuilder.parameterNamePrefix("s" + selectorCount + "p");

        selectorManager.toSql(selector, sqlBuilder);

        if (selectorCount > 0) {
          filterBuilder.append(" or ");
        }

        filterBuilder.append('(').append(sqlBuilder.getQueryString()).append(')');
        filterParameters.putAll(sqlBuilder.getQueryParameters());

        selectorCount++;
      }
      catch (SelectorEvaluationException e) {
        log.warn("Problem evaluating selector {} as SQL", selector.getName(), log.isDebugEnabled() ? e : null);
      }
      finally {
        sqlBuilder.clearQueryString();
      }
    }
  }
}
