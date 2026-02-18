
package org.globalart.astrum.repository.manager.internal;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.distributed.event.service.api.common.RepositoryCacheInvalidationEvent;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryDestroyedEvent;
import org.globalart.astrum.repository.RepositoryEvent;
import org.globalart.astrum.repository.RepositoryStartedEvent;
import org.globalart.astrum.repository.RepositoryStoppedEvent;
import org.globalart.astrum.repository.httpclient.RemoteConnectionStatusEvent;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryLoadedEvent;
import org.globalart.astrum.repository.manager.RepositoryRestoredEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.rest.api.ApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectWriter;
import com.fasterxml.jackson.databind.json.JsonMapper;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Repository auditor.
 *
 * @since 3.1
 */
@Component
public class RepositoryAuditor
    extends AuditorSupport
    implements EventAware
{
  private static final TypeReference<Map<String, Object>> TYPE_REF = new TypeReference<Map<String, Object>>()
  {
  };

  public static final String DOMAIN = "repository";

  private final JsonMapper mapper;

  private final Map<String, ApiRepositoryAdapter> convertersByFormat;

  private final ApiRepositoryAdapter defaultAdapter;

  @Autowired
  public RepositoryAuditor(
      final List<ApiRepositoryAdapter> convertersByFormatList,
      @Qualifier("default") final ApiRepositoryAdapter defaultAdapter,
      final JsonMapper mapper)
  {
    this.convertersByFormat = QualifierUtil.buildQualifierBeanMap(convertersByFormatList);
    this.defaultAdapter = defaultAdapter;
    this.mapper = checkNotNull(mapper);

    registerType(RepositoryCacheInvalidationEvent.class, "cacheInvalidated");
    registerType(RepositoryCreatedEvent.class, CREATED_TYPE);
    registerType(RepositoryRestoredEvent.class, "restored");
    registerType(RepositoryUpdatedEvent.class, UPDATED_TYPE);
    registerType(RepositoryDestroyedEvent.class, "destroyed");
    registerType(RepositoryDeletedEvent.class, DELETED_TYPE);
    registerType(RepositoryLoadedEvent.class, "loaded");
    registerType(RepositoryStartedEvent.class, "started");
    registerType(RepositoryStoppedEvent.class, "stopped");
    registerType(RemoteConnectionStatusEvent.class, "autoBlockStatus");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RepositoryEvent event) {
    if (isRecording()) {
      Repository repository = event.getRepository();
      AuditData data = getAuditData(repository.getName());
      data.setType(type(event.getClass()));

      if (event instanceof RepositoryCreatedEvent || event instanceof RepositoryUpdatedEvent) {
        data.setAttributes(createFullAttributes(repository));
      }
      else {
        Map<String, Object> attributes = createSimple(repository);
        if (event instanceof RemoteConnectionStatusEvent) {
          attributes.put("statusType", ((RemoteConnectionStatusEvent) event).getStatus().getType());
        }
        data.setAttributes(attributes);
      }

      record(data);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RepositoryCacheInvalidationEvent event) {
    if (isRecording()) {
      AuditData data = getAuditData(event.getRepositoryName());
      data.setType(type(event.getClass()));
      record(data);
    }
  }

  private AuditData getAuditData(final String repositoryName) {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setContext(repositoryName);
    return data;
  }

  private Map<String, Object> createFullAttributes(final Repository repository) {
    boolean baseUrlAbsent = !BaseUrlHolder.isSet();
    try {
      if (baseUrlAbsent) {
        BaseUrlHolder.set("", ""); // use empty base URL placeholder during conversion to avoid log-spam
      }

      AbstractApiRepository apiObject = convert(repository);

      ObjectWriter writer = mapper.writerFor(apiObject.getClass());

      String json = writer.writeValueAsString(apiObject);

      return mapper.readerFor(TYPE_REF).readValue(json);
    }
    catch (Exception e) {
      log.error("Failed to convert repo object falling back to simple", e);
      return createSimple(repository);
    }
    finally {
      if (baseUrlAbsent) {
        BaseUrlHolder.unset();
      }
    }
  }

  private Map<String, Object> createSimple(final Repository repository) {
    Map<String, Object> attributes = new HashMap<>();
    attributes.put("name", repository.getName());
    attributes.put("type", repository.getType().getValue());
    attributes.put("format", repository.getFormat().getValue());
    return attributes;

  }

  private AbstractApiRepository convert(final Repository repository) {
    return convertersByFormat.getOrDefault(repository.getFormat().getValue(), defaultAdapter).adapt(repository);
  }
}
