
package org.globalart.astrum.security.usertoken.event;

import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasKey;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class UserTokenAuditorTest
{
  @Mock
  private AuditRecorder auditRecorder;

  private UserTokenAuditor underTest;

  @Before
  public void setup() {
    underTest = new UserTokenAuditor();
    // Use reflection to set the audit recorder since it's protected
    try {
      java.lang.reflect.Field field = underTest.getClass().getSuperclass().getDeclaredField("auditRecorder");
      field.setAccessible(true);
      field.set(underTest, auditRecorder);
    }
    catch (Exception e) {
      throw new RuntimeException("Failed to set audit recorder", e);
    }

    // Enable recording
    when(auditRecorder.isEnabled()).thenReturn(true);
  }

  @Test
  public void testAdminCreatedEvent_extractsAttributes() {
    // Arrange
    UserTokenAdminCreatedEvent event = new UserTokenAdminCreatedEvent(
        "targetUser",
        "AstrumAuthenticatingRealm",
        "adminUser",
        "AstrumAuthorizingRealm",
        "abc123");

    // Act
    underTest.on(event);

    // Assert
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());

    AuditData auditData = captor.getValue();
    assertThat(auditData.getDomain(), is("userToken"));
    assertThat(auditData.getType(), is("adminCreated"));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes, hasKey("targetUserId"));
    assertThat(attributes.get("targetUserId"), is("targetUser"));
    assertThat(attributes, hasKey("targetRealm"));
    assertThat(attributes.get("targetRealm"), is("AstrumAuthenticatingRealm"));
    assertThat(attributes, hasKey("adminUserId"));
    assertThat(attributes.get("adminUserId"), is("adminUser"));
    assertThat(attributes, hasKey("adminRealm"));
    assertThat(attributes.get("adminRealm"), is("AstrumAuthorizingRealm"));
    assertThat(attributes, hasKey("nameCode"));
    assertThat(attributes.get("nameCode"), is("abc123"));
  }

  @Test
  public void testAdminListedEvent_extractsAttributesWithFilters() {
    // Arrange
    UserTokenAdminListedEvent event = new UserTokenAdminListedEvent(
        "adminUser",
        "AstrumAuthorizingRealm",
        "AstrumAuthenticatingRealm",
        "testuser",
        true,
        5);

    // Act
    underTest.on(event);

    // Assert
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());

    AuditData auditData = captor.getValue();
    assertThat(auditData.getDomain(), is("userToken"));
    assertThat(auditData.getType(), is("adminListed"));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes, hasKey("adminUserId"));
    assertThat(attributes.get("adminUserId"), is("adminUser"));
    assertThat(attributes, hasKey("adminRealm"));
    assertThat(attributes.get("adminRealm"), is("AstrumAuthorizingRealm"));
    assertThat(attributes, hasKey("realmFilter"));
    assertThat(attributes.get("realmFilter"), is("AstrumAuthenticatingRealm"));
    assertThat(attributes, hasKey("userIdFilter"));
    assertThat(attributes.get("userIdFilter"), is("testuser"));
    assertThat(attributes, hasKey("includeExpired"));
    assertThat(attributes.get("includeExpired"), is(true));
    assertThat(attributes, hasKey("resultCount"));
    assertThat(attributes.get("resultCount"), is(5));
  }

  @Test
  public void testAdminListedEvent_withNullFilters() {
    // Arrange - no filters provided
    UserTokenAdminListedEvent event = new UserTokenAdminListedEvent(
        "adminUser",
        "AstrumAuthorizingRealm",
        null,
        null,
        false,
        10);

    // Act
    underTest.on(event);

    // Assert
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());

    AuditData auditData = captor.getValue();
    Map<String, Object> attributes = auditData.getAttributes();

    // Should not include null filters
    assertThat(attributes, not(hasKey("realmFilter")));
    assertThat(attributes, not(hasKey("userIdFilter")));

    // Should include non-null values
    assertThat(attributes, hasKey("adminUserId"));
    assertThat(attributes, hasKey("includeExpired"));
    assertThat(attributes, hasKey("resultCount"));
  }

  @Test
  public void testAdminReadEvent_extractsAttributes() {
    // Arrange
    UserTokenAdminReadEvent event = new UserTokenAdminReadEvent(
        "targetUser",
        "AstrumAuthenticatingRealm",
        "adminUser",
        "AstrumAuthorizingRealm");

    // Act
    underTest.on(event);

    // Assert
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());

    AuditData auditData = captor.getValue();
    assertThat(auditData.getDomain(), is("userToken"));
    assertThat(auditData.getType(), is("adminRead"));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes, hasKey("targetUserId"));
    assertThat(attributes.get("targetUserId"), is("targetUser"));
    assertThat(attributes, hasKey("targetRealm"));
    assertThat(attributes.get("targetRealm"), is("AstrumAuthenticatingRealm"));
    assertThat(attributes, hasKey("adminUserId"));
    assertThat(attributes.get("adminUserId"), is("adminUser"));
    assertThat(attributes, hasKey("adminRealm"));
    assertThat(attributes.get("adminRealm"), is("AstrumAuthorizingRealm"));
  }

  @Test
  public void testAdminDeletedEvent_extractsAttributes() {
    // Arrange
    UserTokenAdminDeletedEvent event = new UserTokenAdminDeletedEvent(
        "targetUser",
        "AstrumAuthenticatingRealm",
        "adminUser",
        "AstrumAuthorizingRealm");

    // Act
    underTest.on(event);

    // Assert
    ArgumentCaptor<AuditData> captor = ArgumentCaptor.forClass(AuditData.class);
    verify(auditRecorder).record(captor.capture());

    AuditData auditData = captor.getValue();
    assertThat(auditData.getDomain(), is("userToken"));
    assertThat(auditData.getType(), is("adminDeleted"));

    Map<String, Object> attributes = auditData.getAttributes();
    assertThat(attributes, hasKey("targetUserId"));
    assertThat(attributes.get("targetUserId"), is("targetUser"));
    assertThat(attributes, hasKey("targetRealm"));
    assertThat(attributes.get("targetRealm"), is("AstrumAuthenticatingRealm"));
    assertThat(attributes, hasKey("adminUserId"));
    assertThat(attributes.get("adminUserId"), is("adminUser"));
    assertThat(attributes, hasKey("adminRealm"));
    assertThat(attributes.get("adminRealm"), is("AstrumAuthorizingRealm"));
  }
}
