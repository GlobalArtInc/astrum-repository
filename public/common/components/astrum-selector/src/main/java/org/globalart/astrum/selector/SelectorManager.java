
package org.globalart.astrum.selector;

import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.common.entity.EntityId;

/**
 * Manages content selectors.
 *
 * since 3.1
 */
public interface SelectorManager
    extends Lifecycle
{
  /**
   * Return all existing content selectors.
   */
  List<SelectorConfiguration> browse();

  /**
   * Return all JEXL content selectors.
   *
   * @since 3.6
   */
  default List<SelectorConfiguration> browseJexl() {
    return browse(JexlSelector.TYPE);
  }

  /**
   * Return all content selectors of the given type.
   *
   * @since 3.16
   */
  List<SelectorConfiguration> browse(String selectorType);

  /**
   * @return the content selectors for the active user
   *
   * @since 3.6
   */
  List<SelectorConfiguration> browseActive(Collection<String> repositoryNames, Collection<String> formats);

  /**
   * Read content selector by id.
   *
   * @deprecated use {@link #readByName}
   */
  @Deprecated
  @Nullable
  SelectorConfiguration read(EntityId entityId);

  /**
   * Read content selector by name; unlike {@link #findByName} this will always read from the database.
   *
   * @since 3.21
   */
  @Nullable
  SelectorConfiguration readByName(String name);

  /**
   * Find by name; this may use an intervening cache to avoid repeated reads from the database.
   */
  Optional<SelectorConfiguration> findByName(String name);

  /**
   * Persist a new selector configuration.
   */
  void create(SelectorConfiguration configuration);

  /**
   * @since 3.20
   */
  SelectorConfiguration create(String name, String type, String description, Map<String, String> attributes);

  /**
   * Persist an existing selector configuration.
   */
  void update(SelectorConfiguration configuration);

  /**
   * Delete an existing selector configuration.
   */
  void delete(SelectorConfiguration configuration);

  /**
   * Evaluate the specified content selector against the given variable source.
   *
   * @throws SelectorEvaluationException if the given selector cannot be evaluated
   */
  boolean evaluate(
      SelectorConfiguration selectorConfiguration,
      VariableSource variableSource) throws SelectorEvaluationException;

  /**
   * Convert the specified content selector to SQL for use as a 'where' clause.
   *
   * @throws SelectorEvaluationException if the given selector cannot be converted
   *
   * @since 3.16
   */
  void toSql(
      SelectorConfiguration selectorConfiguration,
      SelectorSqlBuilder sqlBuilder) throws SelectorEvaluationException;

  /**
   * Use the specified CselToSql implementation to convert the selector configuration to SQL for use as a 'where'
   * clause
   */
  <T> void toSql(
      SelectorConfiguration selector,
      T sqlBuilder,
      CselToSql<T> cselToSql) throws SelectorEvaluationException;

  /**
   * @return new instance of SelectorConfiguration
   *
   * @since 3.20
   */
  SelectorConfiguration newSelectorConfiguration();

  /**
   * @return new instance of SelectorConfiguration
   *
   * @since 3.20
   */
  SelectorConfiguration newSelectorConfiguration(
      String name,
      String type,
      String description,
      Map<String, ?> attributes);
}
