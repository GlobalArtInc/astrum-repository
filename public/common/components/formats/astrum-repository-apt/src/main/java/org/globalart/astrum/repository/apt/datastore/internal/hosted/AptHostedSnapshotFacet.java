
package org.globalart.astrum.repository.apt.datastore.internal.hosted;

import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.snapshot.AptSnapshotFacetSupport;
import org.globalart.astrum.repository.apt.internal.snapshot.SnapshotItem;
import org.globalart.astrum.repository.apt.internal.snapshot.SnapshotItem.ContentSpecifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Implementation of snapshots for apt hosted repository.
 *
 * @since 3.31
 */
@Facet.Exposed
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AptHostedSnapshotFacet
    extends AptSnapshotFacetSupport
{
  @Override
  protected List<SnapshotItem> fetchSnapshotItems(final List<ContentSpecifier> specs) {
    AptContentFacet apt = getRepository().facet(AptContentFacet.class);
    List<SnapshotItem> list = new ArrayList<>();
    for (ContentSpecifier spec : specs) {
      apt.get(spec.path).map(value -> new SnapshotItem(spec, value)).ifPresent(list::add);
    }
    return list;
  }
}
