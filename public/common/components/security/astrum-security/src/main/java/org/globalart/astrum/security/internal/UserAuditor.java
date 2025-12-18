
package org.globalart.astrum.security.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditorSupport;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserCreatedEvent;
import org.globalart.astrum.security.user.UserDeletedEvent;
import org.globalart.astrum.security.user.UserEvent;
import org.globalart.astrum.security.user.UserUpdatedEvent;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.stereotype.Component;

/**
 * {@link User} auditor.
 *
 * @since 3.1
 */
@Component
public class UserAuditor
    extends AuditorSupport
    implements EventAware
{
  public static final String DOMAIN = "security.user";

  public UserAuditor() {
    registerType(UserCreatedEvent.class, CREATED_TYPE);
    registerType(UserDeletedEvent.class, DELETED_TYPE);
    registerType(UserUpdatedEvent.class, UPDATED_TYPE);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final UserEvent event) {
    if (isRecording()) {
      User user = event.getUser();

      AuditData data = new AuditData();
      data.setDomain(DOMAIN);
      data.setType(type(event.getClass()));
      data.setContext(user.getUserId());

      Map<String, Object> attributes = data.getAttributes();
      attributes.put("id", user.getUserId());
      attributes.put("name", user.getName());
      attributes.put("email", user.getEmailAddress());
      attributes.put("source", user.getSource());
      attributes.put("status", user.getStatus().name());
      attributes.put("roles", roles(user));

      record(data);
    }
  }

  private static String roles(final User user) {
    List<String> result = new ArrayList<>();

    for (RoleIdentifier role : user.getRoles()) {
      result.add(role.getRoleId());
    }

    return string(result);
  }
}
