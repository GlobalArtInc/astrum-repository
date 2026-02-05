
package org.globalart.astrum.repository.content.kv;

import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.stateguard.Guarded;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.store.InternalIds;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.FacetSupport.State.ATTACHED;
import static org.globalart.astrum.repository.FacetSupport.State.STARTED;

/**
 * Support class encapsulating the key-value store actions.
 */
@Exposed
public abstract class KeyValueFacetSupport<DAO extends KeyValueDAO, STORE extends KeyValueStore<DAO>>
    extends FacetSupport
{
  private final Class<DAO> daoClass;

  private final FormatStoreManager formatStoreManager;

  protected STORE dataStore;

  protected KeyValueFacetSupport(final FormatStoreManager formatStoreManager, final Class<DAO> daoClass) {
    this.formatStoreManager = checkNotNull(formatStoreManager);
    this.daoClass = checkNotNull(daoClass);
  }

  @Override
  protected void doStart() throws Exception {
    ContentFacetSupport contentFacet = (ContentFacetSupport) getRepository().facet(ContentFacet.class);

    String storeName = contentFacet.stores().contentStoreName;

    dataStore = formatStoreManager.formatStore(storeName, daoClass);
  }

  @Override
  protected void doDelete() throws Exception {
    dataStore.removeRepository(repositoryId());
  }

  /**
   * Get the value for the key in the specified category.
   *
   * @param category the category for the property
   * @param key the key for the desired value
   *
   * @return An optional containing the value, or empty if the value is unset.
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected Optional<String> get(final String category, final String key) {
    return dataStore.get(repositoryId(), category, key);
  }

  /**
   * Get the value for the key in the specified category.
   *
   * @param category the category for the property
   * @param key the key for the desired value
   *
   * @return An optional containing the value, or empty if the value is unset.
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected Optional<KeyValue> getKeyValue(final String category, final String key) {
    return dataStore.getKeyValue(repositoryId(), category, key);
  }

  @Guarded(by = {ATTACHED, STARTED})
  public List<KeyValue> findByCategoryAndKeyLike(@Nullable final String category, final String keyLike) {
    return dataStore.findByCategoryAndKeyLike(repositoryId(), category, keyLike);
  }

  /**
   * Set the value for the key in the specified category.
   *
   * @param category the category of this property
   * @param key the key to store the value under
   * @param value the value to store
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected void set(final String category, final String key, final String value) {
    dataStore.set(repositoryId(), category, key, value);
  }

  /**
   * Remove the stored value associated with the key in the specified category if it exists.
   *
   * @param category the category of the key-value pair
   * @param key the key identifying the value
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected void remove(final String category, final String key) {
    dataStore.remove(repositoryId(), category, key);
  }

  /**
   * Remove all data in the specified category for the attached repository.
   *
   * @param category the category to remove associated content from
   */
  @Guarded(by = {ATTACHED, STARTED})
  public void removeAll(final String category) {
    dataStore.removeAll(repositoryId(), category);
  }

  /**
   * Remove all data in the specified category for the attached repository.
   */
  @Guarded(by = {ATTACHED, STARTED})
  public void removeAll() {
    dataStore.removeAll(repositoryId(), null);
  }

  /**
   * Browse all the values stored with a category.
   *
   * @param category the category to browse content for
   * @param limit the page size
   * @param continuationToken the continuation token for the page of results, or null for the first page
   *
   * @return the page of results.
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected Continuation<KeyValue> browseValues(
      final String category,
      final int limit,
      @Nullable final String continuationToken)
  {
    return dataStore.browse(repositoryId(), category, limit, continuationToken);
  }

  /**
   * Browse all the categories for the repository.
   *
   * @return the distinct categories.
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected List<String> browseCategories() {
    return dataStore.browseCategories(repositoryId());
  }

  /**
   * Count all the values stored with a category
   *
   * @param category the category to count the content of
   * @return
   */
  @Guarded(by = {ATTACHED, STARTED})
  public int countValues(final String category) {
    return dataStore.count(repositoryId(), category);
  }

  /**
   * Count all the values stored with a category
   */
  @Guarded(by = {ATTACHED, STARTED})
  public int countValues() {
    return dataStore.count(repositoryId(), null);
  }

  /**
   * Find categories which contain the provided key.
   *
   * @param key the key
   *
   * @return a list of categories
   */
  @Guarded(by = {ATTACHED, STARTED})
  protected List<String> findCategories(final String key) {
    return dataStore.findCategories(repositoryId(), key);
  }

  protected int repositoryId() {
    return InternalIds.contentRepositoryId(getRepository())
        .orElseThrow(IllegalStateException::new);
  }
}
