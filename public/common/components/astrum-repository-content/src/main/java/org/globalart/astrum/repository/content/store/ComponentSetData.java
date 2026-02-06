
package org.globalart.astrum.repository.content.store;

import org.globalart.astrum.common.entity.ContinuationAware;
import org.globalart.astrum.repository.content.ComponentSet;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link ComponentSet} data backed by the content data store.
 */
public class ComponentSetData
    implements ComponentSet, ContinuationAware
{
  protected String namespace;

  protected String name;

  @Override
  public String namespace() {
    return namespace;
  }

  @Override
  public String name() {
    return name;
  }

  // MyBatis setters + validation

  /**
   * Sets the component namespace.
   */
  public void setNamespace(final String namespace) {
    this.namespace = checkNotNull(namespace);
  }

  /**
   * Sets the component name.
   */
  public void setName(final String name) {
    this.name = checkNotNull(name);
  }

  // ContinuationAware

  @Override
  public String nextContinuationToken() {
    return namespace + '|' + name;
  }

  @Override
  public String toString() {
    return "ComponentSet {" +
        "namespace='" + namespace + "'" +
        ", name='" + name + "'" +
        "} " + super.toString();
  }
}
