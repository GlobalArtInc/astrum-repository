
package org.globalart.astrum.repository;

import java.util.function.Predicate;

/**
 * {@link Predicate} that filters repositories according to whether they have a given facet.
 *
 * @since 3.0
 */
public class HasFacet
    implements Predicate<Repository>, com.google.common.base.Predicate<Repository>
{
  private final Class<? extends Facet> type;

  public HasFacet(final Class<? extends Facet> type) {
    this.type = type;
  }

  @Override
  public boolean apply(final Repository input) {
    return test(input);
  }

  @Override
  public boolean test(final Repository input) {
    return input.optionalFacet(type).isPresent();
  }

  public static HasFacet hasFacet(final Class<? extends Facet> type) {
    return new HasFacet(type);
  }
}
