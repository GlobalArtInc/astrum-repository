
package org.globalart.astrum.repository.security;

import java.util.Set;

import javax.annotation.Nullable;

import org.globalart.astrum.selector.VariableSource;

/**
 * Repository content permission checker
 *
 * @since 3.1
 */
public interface ContentPermissionChecker
{
  /**
   * Ensure that either the view permission or the content selector permission is permitted
   */
  boolean isPermitted(
      String repositoryName,
      String repositoryFormat,
      String action,
      VariableSource variableSource);

  /**
   * Ensure that either the view permission or the content selector permission is permitted
   * If any of the actions are permitted, return true
   *
   * @since 3.36
   */
  boolean isPermittedAnyOf(
      String repositoryName,
      String repositoryFormat,
      VariableSource variableSource,
      String... actions);

  /**
   * Ensure that either the view permission or that a JEXL content selector permission is permitted
   *
   * @since 3.6
   */
  boolean isPermittedJexlOnly(
      String repositoryName,
      String repositoryFormat,
      String action,
      VariableSource variableSource);

  /**
   * Ensure that either the view permission or that a JEXL content selector permission is permitted
   * If any of the actions are permitted, return true
   *
   * @since 3.36
   */
  boolean isPermittedJexlOnlyAnyOf(
      String repositoryName,
      String repositoryFormat,
      VariableSource variableSource,
      String... actions);

  /**
   * Ensure that either the view permission or the content selector permission is permitted for the desired
   * repositories
   *
   * @since 3.14
   */
  boolean isPermitted(
      Set<String> repositoryNames,
      String repositoryFormat,
      String action,
      VariableSource variableSource);

  /**
   * Ensure that either the view permission or the content selector permission is permitted for the desired
   * repositories.
   * If any of the actions are permitted, return true
   *
   * @since 3.36
   */
  boolean isPermittedAnyOf(
      Set<String> repositoryNames,
      String repositoryFormat,
      VariableSource variableSource,
      String... actions);

  /**
   * Ensure that either the view permission or the content selector permission is permitted.
   * Uses the provided cache for selector evaluations to improve performance.
   *
   * @param repositoryName repository name
   * @param repositoryFormat repository format
   * @param action action to check
   * @param variableSource variable source for content selector evaluation
   * @param cache optional request-scoped cache for selector evaluations (null to disable caching)
   * @return true if permitted
   */
  boolean isPermitted(
      String repositoryName,
      String repositoryFormat,
      String action,
      VariableSource variableSource,
      @Nullable SelectorEvaluationCache cache);

  /**
   * Ensure that either the view permission or the content selector permission is permitted for the desired
   * repositories.
   * Uses the provided cache for selector evaluations to improve performance.
   *
   * @param repositoryNames repository names
   * @param repositoryFormat repository format
   * @param action action to check
   * @param variableSource variable source for content selector evaluation
   * @param cache optional request-scoped cache for selector evaluations (null to disable caching)
   * @return true if permitted
   */
  boolean isPermitted(
      Set<String> repositoryNames,
      String repositoryFormat,
      String action,
      VariableSource variableSource,
      @Nullable SelectorEvaluationCache cache);

  /**
   * Clears any request-scoped caches.
   * <p>
   * Implementations without caching should provide a no-op implementation.
   * Should be called at request completion to prevent memory leaks.
   * </p>
   * <p>
   * NOTE: This method is deprecated in favor of passing {@link SelectorEvaluationCache}
   * explicitly through the call chain, which provides better lifecycle management.
   * </p>
   *
   * @deprecated Use explicit cache parameter in isPermitted methods instead
   */
  @Deprecated
  default void clearRequestCache() {
    // No-op by default for implementations without caching
  }
}
