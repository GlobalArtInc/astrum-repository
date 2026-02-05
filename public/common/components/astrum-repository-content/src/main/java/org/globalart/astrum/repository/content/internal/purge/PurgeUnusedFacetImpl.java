
package org.globalart.astrum.repository.content.internal.purge;

import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.purge.PurgeUnusedFacet;

import static com.google.common.base.Preconditions.checkArgument;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * @since 3.24
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PurgeUnusedFacetImpl
    extends FacetSupport
    implements PurgeUnusedFacet
{
  @Override
  @Guarded(by = STARTED)
  public void purgeUnused(final int numberOfDays) {
    checkArgument(numberOfDays > 0, "Number of days must be greater then zero");
    log.info("Purging unused components from repository {}", getRepository().getName());

    ContentFacetSupport contentFacet = (ContentFacetSupport) getRepository().facet(ContentFacet.class);
    ComponentStore<?> componentStore = contentFacet.stores().componentStore;
    AssetStore<?> assetStore = contentFacet.stores().assetStore;
    int contentRepositoryId = contentFacet.contentRepositoryId();

    int purgedAssets = assetStore.purgeNotRecentlyDownloaded(contentRepositoryId, numberOfDays);
    log.debug("Deleted {} unused assets without components", purgedAssets);

    int purgedComponents = componentStore.purgeNotRecentlyDownloaded(contentRepositoryId, numberOfDays);
    log.debug("Deleted {} unused components and their assets", purgedComponents);

    log.info("Purged {} unused components from repository {}", purgedComponents, getRepository().getName());
  }
}
