
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.repository.content.Component;

/**
 * {@link Component} data backed by the content data store, that has a dynamic continuation token to allow being
 * returned as part of database queries with differing ORDER BY clauses, and not tied to component ID
 */
public class OrderedComponentData
    extends ComponentData
{

  String orderedToken;

  public OrderedComponentData() {
    // empty constructor to be able to instantiate object using myBatis
  }

  public void setOrderedToken(final String orderedToken) {
    this.orderedToken = orderedToken;
  }

  @Override
  public String nextContinuationToken() {
    return orderedToken;
  }
}
