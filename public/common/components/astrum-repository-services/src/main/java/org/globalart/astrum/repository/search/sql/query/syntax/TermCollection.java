
package org.globalart.astrum.repository.search.sql.query.syntax;

import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

import com.google.common.collect.Iterables;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A collection of {@link StringTerm} which is to the right of the operand in an {@link SqlPredicate}.
 */
public class TermCollection
    implements Term
{
  private final Collection<SingleValueTerm<?>> terms;

  /*
   * To keep things simple we only accept StringTerm to avoid nested TermCollections
   */
  private TermCollection(final Collection<SingleValueTerm<?>> terms) {
    this.terms = List.copyOf(checkNotNull(terms));
  }

  public Collection<SingleValueTerm<?>> get() {
    return Collections.unmodifiableCollection(terms);
  }

  @Override
  public int hashCode() {
    return Objects.hash(terms);
  }

  @Override
  public boolean equals(final Object obj) {
    if (this == obj) {
      return true;
    }
    if (obj == null) {
      return false;
    }
    if (getClass() != obj.getClass()) {
      return false;
    }
    TermCollection other = (TermCollection) obj;
    return Objects.equals(terms, other.terms);
  }

  @Override
  public String toString() {
    return "TermCollection [terms=" + terms + "]";
  }

  /**
   * Helper method to create a Term with the provided collection. If the provided collection has only one element that
   * element will be returned instead of creating an instance.
   */
  public static Term create(final Collection<SingleValueTerm<?>> terms) {
    // We use a private constructor to avoid creating
    if (terms.size() == 1) {
      return Iterables.getOnlyElement(terms);
    }
    return new TermCollection(terms);
  }

  /**
   * Helper method to create a Term with the provided collection. If the provided collection has only one element that
   * element will be returned instead of creating an instance.
   */
  public static Term create(final SingleValueTerm<?>... terms) {
    // We use a private constructor to avoid creating
    if (terms.length == 1) {
      return terms[0];
    }
    return new TermCollection(Arrays.asList(terms));
  }
}
