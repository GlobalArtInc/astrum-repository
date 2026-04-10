
package org.globalart.astrum.repository.apt.datastore.internal.proxy;

import java.io.IOException;
import java.util.List;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.apt.datastore.internal.snapshot.AptSnapshotFacetSupport;
import org.globalart.astrum.repository.apt.internal.snapshot.SnapshotItem;
import org.globalart.astrum.repository.apt.internal.snapshot.SnapshotItem.ContentSpecifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Implementation of snapshots for apt proxy repository.
 *
 * @since 3.31
 */
@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AptProxySnapshotFacet
    extends AptSnapshotFacetSupport
{
  @Override
  protected List<SnapshotItem> fetchSnapshotItems(final List<ContentSpecifier> specs) throws IOException {
    AptProxyFacet proxyFacet = getRepository().facet(AptProxyFacet.class);
    return proxyFacet.getSnapshotItems(specs);
  }
}
