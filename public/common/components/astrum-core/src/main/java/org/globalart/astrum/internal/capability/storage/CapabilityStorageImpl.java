
package org.globalart.astrum.internal.capability.storage;

import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.common.entity.EntityId;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.common.entity.HasEntityId;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.internal.capability.storage.datastore.CapabilityStorageItemCreatedEventImpl;
import org.globalart.astrum.internal.capability.storage.datastore.CapabilityStorageItemDeletedEventImpl;
import org.globalart.astrum.internal.capability.storage.datastore.CapabilityStorageItemUpdatedEventImpl;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.collect.ImmutableMap.toImmutableMap;
import static com.google.common.collect.Streams.stream;
import static java.util.UUID.fromString;
import static java.util.function.Function.identity;

/**
 * MyBatis {@link CapabilityStorage} implementation.
 *
 * @since 3.21
 */
@Component
@Qualifier("mybatis")
public class CapabilityStorageImpl
    extends ConfigStoreSupport<CapabilityStorageItemDAO>
    implements CapabilityStorage
{
  @Autowired
  public CapabilityStorageImpl(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Override
  public CapabilityStorageItem newStorageItem(
      final int version,
      final String type,
      final boolean enabled,
      final String notes,
      final Map<String, String> properties)
  {
    CapabilityStorageItem item = new CapabilityStorageItemData();
    item.setVersion(version);
    item.setType(type);
    item.setEnabled(enabled);
    item.setNotes(notes);
    item.setProperties(properties);
    return item;
  }

  @Transactional
  @Override
  public CapabilityIdentity add(final CapabilityStorageItem item) {
    postCommitEvent(() -> new CapabilityStorageItemCreatedEventImpl((CapabilityStorageItemData) item));
    try {
      dao().create((CapabilityStorageItemData) item);
    }
    catch (DuplicateKeyException e) {
      log.debug("Trying to add duplicate for {} capability. Ignore it.", item);
    }
    return capabilityIdentity(item);
  }

  @Transactional
  @Override
  public boolean update(final CapabilityIdentity id, final CapabilityStorageItem item) {
    postCommitEvent(() -> new CapabilityStorageItemUpdatedEventImpl((CapabilityStorageItemData) item));
    ((HasEntityId) item).setId(entityId(id));
    return dao().update((CapabilityStorageItemData) item);
  }

  @Transactional
  @Override
  public boolean remove(final CapabilityIdentity id) {
    getAll().values()
        .stream()
        .filter(capability -> id.equals(capabilityIdentity(capability)))
        .findFirst()
        .map(CapabilityStorageItemData.class::cast)
        .ifPresent(item -> postCommitEvent(() -> new CapabilityStorageItemDeletedEventImpl(item)));
    return dao().delete(entityId(id));
  }

  @Transactional
  @Override
  public Optional<CapabilityStorageItem> read(final EntityId entityId) {
    return dao().read(entityId).map(CapabilityStorageItemData.class::cast);
  }

  @Transactional
  @Override
  public Map<CapabilityIdentity, CapabilityStorageItem> getAll() {
    return stream(dao().browse()).collect(toImmutableMap(CapabilityStorageImpl::capabilityIdentity, identity()));
  }

  public static CapabilityIdentity capabilityIdentity(final CapabilityStorageItem item) {
    return new CapabilityIdentity(((HasEntityId) item).getId().getValue());
  }

  private static EntityId entityId(final CapabilityIdentity id) {
    return new EntityUUID(fromString(id.toString()));
  }
}
