
package org.globalart.astrum.repository.content.fluent.constraints;

import java.util.Set;

import org.globalart.astrum.repository.Repository;

/**
 * Will add constraints to the fluent query, currently only exposing repository level constraints
 */
public interface FluentQueryConstraint
{
  Set<Integer> getRepositoryIds(final Repository repository);
}
