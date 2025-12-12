
package org.globalart.astrum.security.usertoken.event;

import java.util.Map;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * UserToken auditor.
 *
 */
@Component
public class UserTokenAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "userToken";

  @Autowired
  public UserTokenAuditor() {
    registerType(UserTokenEvent.class, CREATED_TYPE);
    registerType(UserTokenDeletedEvent.class, DELETED_TYPE);
    registerType(UserTokenPurgedEvent.class, PURGE_TYPE);
    registerType(UserTokenConfigChangedEvent.class, "configChanged");
    registerType(UserTokenAdminCreatedEvent.class, "adminCreated");
    registerType(UserTokenAdminReadEvent.class, "adminRead");
    registerType(UserTokenAdminListedEvent.class, "adminListed");
    registerType(UserTokenAdminDeletedEvent.class, "adminDeleted");
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserTokenEvent event) {
    if (isRecording()) {
      AuditData data = getAuditData(event.getEventType().toString());
      if (event instanceof UserTokenConfigChangedEvent) {
        Map<String, Object> attributes = data.getAttributes();
        UserTokenConfigChangedEvent configChangedEvent = (UserTokenConfigChangedEvent) event;
        attributes.put("Enabled", configChangedEvent.isEnabled());
        attributes.put("Protect", configChangedEvent.isProtectContent());
        if (configChangedEvent.isExpirationEnabled()) {
          attributes.put("Expiration Days", configChangedEvent.getExpirationDays());
        }
      }
      else if (event instanceof UserTokenAdminCreatedEvent) {
        UserTokenAdminCreatedEvent adminEvent = (UserTokenAdminCreatedEvent) event;
        Map<String, Object> attributes = data.getAttributes();
        attributes.put("targetUserId", adminEvent.getTargetUserId());
        attributes.put("targetRealm", adminEvent.getTargetRealm());
        attributes.put("adminUserId", adminEvent.getAdminUserId());
        attributes.put("adminRealm", adminEvent.getAdminRealm());
        attributes.put("nameCode", adminEvent.getNameCode());
      }
      else if (event instanceof UserTokenAdminReadEvent) {
        UserTokenAdminReadEvent adminEvent = (UserTokenAdminReadEvent) event;
        Map<String, Object> attributes = data.getAttributes();
        attributes.put("targetUserId", adminEvent.getTargetUserId());
        attributes.put("targetRealm", adminEvent.getTargetRealm());
        attributes.put("adminUserId", adminEvent.getAdminUserId());
        attributes.put("adminRealm", adminEvent.getAdminRealm());
      }
      else if (event instanceof UserTokenAdminListedEvent) {
        UserTokenAdminListedEvent adminEvent = (UserTokenAdminListedEvent) event;
        Map<String, Object> attributes = data.getAttributes();
        attributes.put("adminUserId", adminEvent.getAdminUserId());
        attributes.put("adminRealm", adminEvent.getAdminRealm());
        if (adminEvent.getRealmFilter() != null) {
          attributes.put("realmFilter", adminEvent.getRealmFilter());
        }
        if (adminEvent.getUserIdFilter() != null) {
          attributes.put("userIdFilter", adminEvent.getUserIdFilter());
        }
        attributes.put("includeExpired", adminEvent.isIncludeExpired());
        attributes.put("resultCount", adminEvent.getResultCount());
      }
      else if (event instanceof UserTokenAdminDeletedEvent) {
        UserTokenAdminDeletedEvent adminEvent = (UserTokenAdminDeletedEvent) event;
        Map<String, Object> attributes = data.getAttributes();
        attributes.put("targetUserId", adminEvent.getUsername());
        attributes.put("targetRealm", adminEvent.getTargetRealm());
        attributes.put("adminUserId", adminEvent.getAdminUserId());
        attributes.put("adminRealm", adminEvent.getAdminRealm());
      }
      else if (event instanceof UserTokenDeletedEvent) {
        UserTokenDeletedEvent deletedEvent = (UserTokenDeletedEvent) event;
        int deleted = deletedEvent.getDeleted();
        String username = deletedEvent.getUsername();
        Map<String, Object> attributes = data.getAttributes();
        attributes.put("Deleted", deleted);
        if (null != username) {
          attributes.put("username", username);
        }
      }
      data.setType(type(event.getClass()));

      record(data);
    }
  }

  private AuditData getAuditData(final String eventType) {
    AuditData data = new AuditData();
    data.setDomain(DOMAIN);
    data.setContext(eventType);
    return data;
  }

}
