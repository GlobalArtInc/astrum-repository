
package org.globalart.astrum.repository.content.utils;

import java.util.stream.Collectors;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.fluent.FluentComponent;

/**
 * A format specifc evaluator to determine whether the component indicates a pre-release version.
 * Implementations are format specific.
 *
 * Note: an implementor which is not concerned with a component's assets should override both methods which may avoid a
 * database query.
 *
 * @since 3.38
 */
public interface PreReleaseEvaluator
{
  /**
   * Evaluates whether the component or its assets are considered to be a pre-release version.
   * Implementations are format specific.
   */
  default boolean isPreRelease(final FluentComponent component) {
    return isPreRelease(component, component.assets().stream().map(Asset.class::cast).collect(Collectors.toList()));
  }

  /**
   * Evaluates whether the component or the assets are considered to be a pre-release version.
   * Implementations are format specific.
   */
  boolean isPreRelease(Component component, Iterable<Asset> assets);
}
